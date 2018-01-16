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

const protos = require('../blockchain/protos')
const {
  awaitServerPubkey,
  getTxnCreator,
  submitTxns,
  encodeTimestampedPayload
} = require('../system/submit_utils')

const DATA = process.env.DATA
if (DATA.indexOf('.json') === -1) {
  throw new Error('Use the "DATA" environment variable to specify a JSON file')
}

const types = require(`./${DATA}`)

protos.compile()
  .then(awaitServerPubkey)
  .then(batcherPublicKey => getTxnCreator(null, batcherPublicKey))
  .then(createTxn => {
    const agentPayload = encodeTimestampedPayload({
      action: protos.SCPayload.Action.CREATE_AGENT,
      createAgent: protos.CreateAgentAction.create({
        name: 'Supply Chain Admin'
      })
    })

    const typePayloads = types.map(type => {
      return encodeTimestampedPayload({
        action: protos.SCPayload.Action.CREATE_RECORD_TYPE,
        createRecordType: protos.CreateRecordTypeAction.create({
          name: type.name,
          properties: type.properties.map(prop => {
            return protos.PropertySchema.create(prop)
          })
        })
      })
    })

    const txns = [ createTxn(agentPayload) ]
      .concat(typePayloads.map(payload => createTxn(payload)))
    return submitTxns(txns)
  })
  .then(res => console.log('Types submitted:\n', JSON.parse(res)))
  .catch(err => {
    console.error(err.toString())
    process.exit()
  })
