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

const request = require('request-promise-native')
const { signer, TransactionEncoder } = require('sawtooth-sdk')
const protos = require('../blockchain/protos')

const SERVER = process.env.SERVER || 'http://localhost:3000'
const DATA = process.env.DATA

if (DATA.indexOf('.json') === -1) {
  throw new Error('Use the "DATA" environment variable to specify a JSON file')
}

const types = require(`./${DATA}`)

protos.compile()
  .then(() => request(`${SERVER}/api/info`))
  .then(res => JSON.parse(res).pubkey)
  .then(batcherPubkey => {
    const privateKey = signer.makePrivateKey()
    return new TransactionEncoder(privateKey, {
      familyName: 'supply_chain',
      familyVersion: '1.0',
      payloadEncoding: 'application/protobuf',
      inputs: ['3400de'],
      outputs: ['3400de'],
      batcherPubkey
    })
  })
  .then(encoder => {
    const agentPayload = protos.SCPayload.encode({
      action: protos.SCPayload.Action.CREATE_AGENT,
      timestamp: Math.floor(Date.now() / 1000),
      createAgent: protos.CreateAgentAction.create({
        name: 'Supply Chain Admin'
      })
    }).finish()

    const typePayloads = types.map(type => {
      return protos.SCPayload.encode({
        action: protos.SCPayload.Action.CREATE_RECORD_TYPE,
        timestamp: Math.floor(Date.now() / 1000),
        createRecordType: protos.CreateRecordTypeAction.create({
          name: type.name,
          properties: type.properties.map(prop => {
            return protos.PropertySchema.create(prop)
          })
        })
      }).finish()
    })

    const txns = [ encoder.create(agentPayload) ]
      .concat(typePayloads.map(payload => encoder.create(payload)))

    return request({
      method: 'POST',
      url: `${SERVER}/api/transactions`,
      headers: { 'Content-Type': 'application/octet-stream' },
      encoding: null,
      body: encoder.encode(txns)
    })
    .catch(err => {
      console.error(err.response.body.toString())
      process.exit()
    })
  })
  .then(res => console.log(JSON.parse(res)))
