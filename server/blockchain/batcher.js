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

const secp256k1 = require('sawtooth-sdk/signing/secp256k1')
const {
  Batch,
  BatchHeader,
  TransactionHeader,
  TransactionList
} = require('sawtooth-sdk/protobuf')
const { BadRequest } = require('../api/errors')
const config = require('../system/config')

const PRIVATE_KEY = config.PRIVATE_KEY

// Initialize secp256k1 Context and PrivateKey wrappers
const context = new secp256k1.Secp256k1Context()
const privateKey = secp256k1.Secp256k1PrivateKey.fromHex(PRIVATE_KEY)
const publicKeyHex = context.getPublicKey(privateKey).asHex()
console.log(`Batch signer initialized with public key: ${publicKeyHex}`)

// Decode transaction headers and throw errors if invalid
const validateTxns = txns => {
  const headers = txns.map(txn => TransactionHeader.decode(txn.header))

  headers.forEach(header => {
    if (header.batcherPublicKey !== publicKeyHex) {
      throw new BadRequest(
        `Transactions must use batcherPublicKey: ${publicKeyHex}`)
    }
  })
}

// Wrap an array of transactions in an encoded BatchList
const batchTxns = txns => {
  const header = BatchHeader.encode({
    signerPublicKey: publicKeyHex,
    transactionIds: txns.map(txn => txn.headerSignature)
  }).finish()

  return Batch.create({
    header,
    headerSignature: context.sign(header, privateKey),
    transactions: txns
  })
}

// Validate an encoded TransactionList, then wrap in an encoded BatchList
const batch = txnList => {
  const txns = TransactionList.decode(txnList).transactions
  validateTxns(txns)
  return batchTxns(txns)
}

// Return the server's hex encoded public key
const getPublicKey = () => publicKeyHex

module.exports = {
  batch,
  getPublicKey
}
