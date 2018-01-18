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
const protos = require('../blockchain/protos')
const {
  awaitServerPubkey,
  getTxnCreator,
  submitTxns,
  encodeTimestampedPayload
} = require('../system/submit_utils')

const SERVER = process.env.SERVER || 'http://localhost:3000'
const DATA = process.env.DATA
if (DATA.indexOf('.json') === -1) {
  throw new Error('Use the "DATA" environment variable to specify a JSON file')
}

const { records, agents } = require(`./${DATA}`)
let createTxn = null

const createProposal = (privateKey, action) => {
  return createTxn(privateKey, encodeTimestampedPayload({
    action: protos.SCPayload.Action.CREATE_PROPOSAL,
    createProposal: protos.CreateProposalAction.create(action)
  }))
}

const answerProposal = (privateKey, action) => {
  return createTxn(privateKey, encodeTimestampedPayload({
    action: protos.SCPayload.Action.ANSWER_PROPOSAL,
    answerProposal: protos.AnswerProposalAction.create(action)
  }))
}

protos.compile()
  .then(awaitServerPubkey)
  .then(batcherPublicKey => {
    const txnCreators = {}

    createTxn = (privateKey, payload) => {
      if (!txnCreators[privateKey]) {
        txnCreators[privateKey] = getTxnCreator(privateKey, batcherPublicKey)
      }
      return txnCreators[privateKey](payload)
    }
  })

  // Create Agents
  .then(() => {
    console.log('Creating Agents . . .')
    const agentAdditions = agents.map(agent => {
      return createTxn(agent.privateKey, encodeTimestampedPayload({
        action: protos.SCPayload.Action.CREATE_AGENT,
        createAgent: protos.CreateAgentAction.create({ name: agent.name })
      }))
    })

    return submitTxns(agentAdditions)
  })

  // Create Users
  .then(() => {
    console.log('Creating Users . . .')
    const userRequests = agents.map(agent => {
      const user = _.omit(agent, 'name', 'privateKey', 'hashedPassword')
      user.password = agent.hashedPassword
      return request({
        method: 'POST',
        url: `${SERVER}/users`,
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(user)
      })
    })

    return Promise.all(userRequests)
  })

  // Create Records
  .then(() => {
    console.log('Creating Records . . .')
    const recordAdditions = records.map(record => {
      const properties = record.properties.map(property => {
        if (property.dataType === protos.PropertySchema.DataType.LOCATION) {
          property.locationValue = protos.Location.create(property.locationValue)
        }
        return protos.PropertyValue.create(property)
      })

      return createTxn(agents[record.ownerIndex || 0].privateKey, encodeTimestampedPayload({
        action: protos.SCPayload.Action.CREATE_RECORD,
        createRecord: protos.CreateRecordAction.create({
          recordId: record.recordId,
          recordType: record.recordType,
          properties
        })
      }))
    })

    return submitTxns(recordAdditions)
  })

  // Transfer Custodianship
  .then(() => {
    console.log('Transferring Custodianship . . .')
    const custodianProposals = records
      .filter(record => record.custodianIndex !== undefined)
      .map(record => {
        return createProposal(agents[record.ownerIndex || 0].privateKey, {
          recordId: record.recordId,
          receivingAgent: agents[record.custodianIndex].publicKey,
          role: protos.Proposal.Role.CUSTODIAN
        })
      })

    return submitTxns(custodianProposals)
  })
  .then(() => {
    const custodianAnswers = records
      .filter(record => record.custodianIndex !== undefined)
      .map(record => {
        return answerProposal(agents[record.custodianIndex].privateKey, {
          recordId: record.recordId,
          receivingAgent: agents[record.custodianIndex].publicKey,
          role: protos.Proposal.Role.CUSTODIAN,
          response: protos.AnswerProposalAction.Response.ACCEPT
        })
      })

    return submitTxns(custodianAnswers)
  })

  // Authorize New Reporters
  .then(() => {
    console.log('Authorizing New Reporters . . .')
    const reporterProposals = records
      .filter(record => record.reporterIndex !== undefined)
      .map(record => {
        return createProposal(agents[record.ownerIndex || 0].privateKey, {
          recordId: record.recordId,
          receivingAgent: agents[record.reporterIndex].publicKey,
          role: protos.Proposal.Role.REPORTER,
          properties: record.reportableProperties
        })
      })

    return submitTxns(reporterProposals)
  })
  .then(() => {
    const reporterAnswers = records
      .filter(record => record.reporterIndex !== undefined)
      .map(record => {
        return answerProposal(agents[record.reporterIndex].privateKey, {
          recordId: record.recordId,
          receivingAgent: agents[record.reporterIndex].publicKey,
          role: protos.Proposal.Role.REPORTER,
          response: protos.AnswerProposalAction.Response.ACCEPT
        })
      })

    return submitTxns(reporterAnswers)
  })
  .catch(err => {
    console.error(err.toString())
    process.exit()
  })
