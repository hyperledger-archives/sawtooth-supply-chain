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
const FAMILY_VERSION = '1.1'
const NAMESPACE = '3400de'

const SERVER = process.env.SERVER || 'http://localhost:3000'
const RETRY_WAIT = process.env.RETRY_WAIT || 5000

const awaitServerInfo = () => {
  return request(`${SERVER}/info`)
    .catch(() => {
      console.warn(
        `Server unavailable, retrying in ${RETRY_WAIT / 1000} seconds...`)
      return new Promise(resolve => setTimeout(resolve, RETRY_WAIT))
        .then(awaitServerInfo)
    })
}

const awaitServerPubkey = () => {
  return awaitServerInfo().then(info => JSON.parse(info).pubkey)
}

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

const getTxnCreator = (privateKeyHex = null, batcherPublicKeyHex = null) => {
  const context = new secp256k1.Secp256k1Context()
  const privateKey = privateKeyHex === null
    ? context.newRandomPrivateKey()
    : secp256k1.Secp256k1PrivateKey.fromHex(privateKeyHex)

  const signerPublicKey = context.getPublicKey(privateKey).asHex()
  const batcherPublicKey = batcherPublicKeyHex === null
    ? signerPublicKey
    : batcherPublicKeyHex

  return payload => {
    const header = encodeHeader(signerPublicKey, batcherPublicKey, payload)
    const headerSignature = context.sign(header, privateKey)
    return Transaction.create({ header, headerSignature, payload })
  }
}

const submitTxns = transactions => {
  return request({
    method: 'POST',
    url: `${SERVER}/transactions?wait`,
    headers: { 'Content-Type': 'application/octet-stream' },
    encoding: null,
    body: TransactionList.encode({ transactions }).finish()
  })
}

const encodeTimestampedPayload = message => {
  return protos.SCPayload.encode(_.assign({
    timestamp: Math.floor(Date.now() / 1000)
  }, message)).finish()
}

module.exports = {
  awaitServerPubkey,
  getTxnCreator,
  submitTxns,
  encodeTimestampedPayload
}
