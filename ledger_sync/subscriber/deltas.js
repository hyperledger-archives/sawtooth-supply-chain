/**
 * Copyright 2018 Intel Corporation
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
const blocks = require('../db/blocks')
const state = require('../db/state')
const protos = require('./protos')

const deltaQueue = {
  _queue: [],
  _running: false,

  add (promisedFn) {
    this._queue.push(promisedFn)
    this._runUntilEmpty()
  },

  _runUntilEmpty () {
    if (this._running) return
    this._running = true
    this._runNext()
  },

  _runNext () {
    if (this._queue.length === 0) {
      this._running = false
    } else {
      const current = this._queue.shift()
      return current().then(() => this._runNext())
    }
  }
}

const getProtoName = address => {
  const typePrefix = address.slice(6, 8)
  if (typePrefix === 'ea') {
    if (address.slice(-4) === '0000') return 'Property'
    else return 'PropertyPage'
  }

  const names = {
    ae: 'Agent',
    aa: 'Proposal',
    ec: 'Record',
    ee: 'RecordType'
  }
  if (names[typePrefix]) return names[typePrefix]

  throw new Error(`Blockchain Error: No Protobuf for prefix "${typePrefix}"`)
}

const getObjectifier = address => {
  const name = getProtoName(address)
  return stateInstance => {
    const obj = protos[name].toObject(stateInstance, {
      enums: String,  // use string names for enums
      longs: Number,  // convert int64 to Number, limiting precision to 2^53
      defaults: true  // use default for falsey values
    })
    if (name === 'PropertyPage') {
      obj.pageNum = parseInt(address.slice(-4), 16)
    }
    return obj
  }
}

const stateAdder = address => {
  const addState = state[`add${getProtoName(address)}`]
  const toObject = getObjectifier(address)
  return (stateInstance, blockNum) => {
    addState(toObject(stateInstance), blockNum)
  }
}

const getEntries = ({ address, value }) => {
  return protos[`${getProtoName(address)}Container`]
    .decode(value)
    .entries
}

const entryAdder = block => change => {
  const addState = stateAdder(change.address)
  return Promise.all(getEntries(change).map(entry => {
    return addState(entry, block.blockNum)
  }))
}

const handle = (block, changes) => {
  deltaQueue.add(() => {
    const [ pageChanges, otherChanges ] = _.partition(changes, change => {
      return getProtoName(change.address) === 'PropertyPage'
    })

    return Promise.all(otherChanges.map(entryAdder(block)))
      .then(() => {
        // If there are page changes, give other changes a chance to propagate
        const wait = pageChanges.length === 0 ? 0 : 100
        return new Promise(resolve => setTimeout(resolve, wait))
      })
      .then(() =>  Promise.all(pageChanges.map(entryAdder(block))))
      .then(() => blocks.insert(block))
  })
}

module.exports = {
  handle
}
