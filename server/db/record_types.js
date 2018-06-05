/**
 * Copyright 2018 Cargill Incorporated
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

const r = require('rethinkdb')
const db = require('./')

// Returns true if a resource is included in the block with the passed number
const fromBlock = blockNum => resource => {
  return r.and(
    resource('startBlockNum').le(blockNum),
    resource('endBlockNum').gt(blockNum))
}

// Transforms an array of resources with a "name" property
// to an object where names are the keys
const arrayToObject = namedResources => {
  return r.object(r.args(namedResources.concatMap(resource => {
    return [ resource('name'), resource.without('name') ]
  })))
}

// Transforms raw recordType entity into the publishable form the API expects
const publishRecordType = type => {
  return r.expr({
    name: type('name'),
    properties: arrayToObject(type('properties'))
  })
}

const fetchQuery = name => currentBlock => {
  return r.table('recordTypes')
    .getAll(name, { index: 'name' })
    .filter(fromBlock(currentBlock))
    .map(publishRecordType)
    .nth(0)
    .default(null)
}

const listQuery = filterQuery => currentBlock => {
  return r.table('recordTypes')
    .filter(fromBlock(currentBlock))
    .filter(filterQuery)
    .map(publishRecordType)
    .coerceTo('array')
}

const fetch = name => db.queryWithCurrentBlock(fetchQuery(name))

const list = filterQuery => db.queryWithCurrentBlock(listQuery(filterQuery))

module.exports = {
  fetch,
  list
}
