/**
 * Copyright 2017 Intel Corporation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * ----------------------------------------------------------------------------
 */
'use strict'

const _ = require('lodash')
const { Stream } = require('sawtooth-sdk/messaging/stream')
const {
  Message,
  EventList,
  EventSubscription,
  EventFilter,
  StateChangeList,
  ClientEventsSubscribeRequest,
  ClientEventsSubscribeResponse,
  ClientBatchSubmitRequest,
  ClientBatchSubmitResponse,
  ClientBatchStatus,
  ClientBatchStatusRequest,
  ClientBatchStatusResponse
} = require('sawtooth-sdk/protobuf')

const deltas = require('./deltas')
const batcher = require('./batcher')
const config = require('../system/config')

const PREFIX = '3400de'
const NULL_BLOCK_ID = '0000000000000000'
const VALIDATOR_URL = config.VALIDATOR_URL
const stream = new Stream(VALIDATOR_URL)

const getBlock = events => {
  const block = _.chain(events)
    .find(e => e.eventType === 'sawtooth/block-commit')
    .get('attributes')
    .map(a => [a.key, a.value])
    .fromPairs()
    .value()

  return {
    blockNum: parseInt(block.block_num),
    blockId: block.block_id,
    stateRootHash: block.state_root_hash
  }
}

const getChanges = events => {
  const event = events.find(e => e.eventType === 'sawtooth/state-delta')
  if (!event) return []

  const changeList = StateChangeList.decode(event.data)
  return changeList.stateChanges
    .filter(change => change.address.slice(0, 6) === PREFIX)
}

const subscribe = () => {
  return new Promise((resolve, reject) => {
    stream.connect(() => {
      // Set up onReceive handlers
      stream.onReceive(msg => {
        if (msg.messageType === Message.MessageType.CLIENT_EVENTS) {
          const events = EventList.decode(msg.content).events
          deltas.handle(getBlock(events), getChanges(events))
        } else {
          console.error('Received message of unknown type:', msg.messageType)
        }
      })

      // Send subscribe request
      const blockSub = EventSubscription.create({
        eventType: 'sawtooth/block-commit'
      })
      const deltaSub = EventSubscription.create({
        eventType: 'sawtooth/state-delta',
        filters: [EventFilter.create({
          key: 'address',
          matchString: `^${PREFIX}.*`,
          filterType: EventFilter.FilterType.REGEX_ANY
        })]
      })

      stream.send(
        Message.MessageType.CLIENT_EVENTS_SUBSCRIBE_REQUEST,
        ClientEventsSubscribeRequest.encode({
          lastKnownBlockIds: [NULL_BLOCK_ID],
          subscriptions: [blockSub, deltaSub]
        }).finish()
      )
        .then(response => ClientEventsSubscribeResponse.decode(response))
        .then(decoded => {
          const status = _.findKey(ClientEventsSubscribeResponse.Status,
                                   val => val === decoded.status)
          if (status !== 'OK') {
            throw new Error(`Validator responded with status "${status}"`)
          }
        })
        .then(() => resolve())
        .catch(err => reject(err))
    })
  })
}

const submit = (txnBytes, { wait }) => {
  const batch = batcher.batch(txnBytes)

  return stream.send(
    Message.MessageType.CLIENT_BATCH_SUBMIT_REQUEST,
    ClientBatchSubmitRequest.encode({
      batches: [batch]
    }).finish()
  )
  .then(response => ClientBatchSubmitResponse.decode(response))
  .then((decoded) => {
    const submitStatus = _.findKey(ClientBatchSubmitResponse.Status,
                             val => val === decoded.status)
    if (submitStatus !== 'OK') {
      throw new Error(`Batch submission failed with status '${submitStatus}'`)
    }

    if (wait === null) {
      return { batch: batch.headerSignature }
    }

    return stream.send(
      Message.MessageType.CLIENT_BATCH_STATUS_REQUEST,
      ClientBatchStatusRequest.encode({
        batchIds: [batch.headerSignature],
        wait: true,
        timeout: wait
      }).finish()
    )
    .then(statusResponse => {
      const statusBody = ClientBatchStatusResponse
        .decode(statusResponse)
        .batchStatuses[0]

      if (statusBody.status !== ClientBatchStatus.Status.COMMITTED) {
        const id = statusBody.batchId
        const status = _.findKey(ClientBatchStatus.Status,
                                 val => val === statusBody.status)
        const message = statusBody.invalidTransactions.length > 0
          ? statusBody.invalidTransactions[0].message
          : ''
        throw new Error(`Batch ${id} is ${status}, with message: ${message}`)
      }

      // Wait to return until new block is in database
      return new Promise(resolve => setTimeout(() => {
        resolve({ batch: batch.headerSignature })
      }, 1000))
    })
  })
}

module.exports = {
  subscribe,
  submit
}
