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
const { createHash } = require('crypto')
const secp256k1 = require('sawtooth-sdk/signing/secp256k1')
const {
  Transaction,
  TransactionHeader,
  TransactionList
} = require('sawtooth-sdk/protobuf')
const protos = require('../blockchain/protos')

const FAMILY_NAME = 'supply_chain'
const FAMILY_VERSION = '1.0'
const NAMESPACE = '3400de'

const SERVER = process.env.SERVER || 'http://localhost:3000'
const DATA = process.env.DATA

if (DATA.indexOf('.json') === -1) {
  throw new Error('Use the "DATA" environment variable to specify a JSON file')
}

const types = require(`./${DATA}`)

const encodeHeader = (signerPublicKey, batcherPublicKey, payload) => {
  return TransactionHeader.encode({
    signerPublicKey,
    batcherPublicKey,
    familyName: FAMILY_NAME,
    familyVersion: FAMILY_VERSION,
    inputs: [NAMESPACE],
    outputs: [NAMESPACE],
    nonce: (Math.random() * 10 ** 18).toString(36),
    payloadSha512: createHash('sha512').update(payload).digest('hex')
  }).finish()
}

const txnCreator = batcherPublicKey => {
  const context = new secp256k1.Secp256k1Context()
  const privateKey = context.newRandomPrivateKey()
  const signerPublicKey = context.getPublicKey(privateKey).asHex()

  return payload => {
    const header = encodeHeader(signerPublicKey, batcherPublicKey, payload)
    const headerSignature = context.sign(header, privateKey)
    return Transaction.create({ header, headerSignature, payload })
  }
}

protos.compile()
  .then(() => request(`${SERVER}/api/info`))
  .then(res => JSON.parse(res).pubkey)
  .then(batcherPublicKey => txnCreator(batcherPublicKey))
  .then(createTxn => {
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

    const transactions = [ createTxn(agentPayload) ]
      .concat(typePayloads.map(payload => createTxn(payload)))

    return request({
      method: 'POST',
      url: `${SERVER}/api/transactions`,
      headers: { 'Content-Type': 'application/octet-stream' },
      encoding: null,
      body: TransactionList.encode({ transactions }).finish()
    })
    .catch(err => {
      console.error(err.error.toString())
      process.exit()
    })
  })
  .then(res => console.log(JSON.parse(res)))
