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
const r = require('rethinkdb')
const db = require('./')


const valueNames = {
  BYTES: 'bytesValue',
  BOOLEAN: 'booleanValue',
  NUMBER: 'numberValue',
  STRING: 'stringValue',
  ENUM: 'enumValue',
  LOCATION: 'locationValue'
}

const xformStruct = properties => {
  return _.fromPairs(properties.map(property => {
    const value = property.dataType === 'STRUCT'
      ? xformStruct(property.structValues)
      : property[ valueNames[property.dataType] ]
    return [property.name, value]
  }))
}

const addBlockState = (tableName, indexName, indexValue, doc, blockNum) => {
  return db.modifyTable(tableName, table => {
    return table
      .getAll(indexValue, { index: indexName })
      .filter({ endBlockNum: Number.MAX_SAFE_INTEGER })
      .coerceTo('array')
      .do(oldDocs => {
        return oldDocs
          .filter({ startBlockNum: blockNum })
          .coerceTo('array')
          .do(duplicates => {
            return r.branch(
              // If there are duplicates, do nothing
              duplicates.count().gt(0),
              duplicates,

              // Otherwise, update the end block on any old docs,
              // and insert the new one
              table
                .getAll(indexValue, { index: indexName })
                .update({ endBlockNum: blockNum })
                .do(() => {
                  return table.insert(_.assign({}, doc, {
                    startBlockNum: blockNum,
                    endBlockNum: Number.MAX_SAFE_INTEGER
                  }))
                })
            )
          })
      })
  })
}

const addAgent = (agent, blockNum) => {
  return addBlockState('agents', 'publicKey', agent.publicKey,
                       agent, blockNum)
}

const addRecord = (record, blockNum) => {
  return addBlockState('records', 'recordId', record.recordId,
                       record, blockNum)
}

const addRecordType = (type, blockNum) => {
  return addBlockState('recordTypes', 'name', type.name,
                       type, blockNum)
}

const addProperty = (property, blockNum) => {
  return addBlockState('properties', 'attributes',
                       ['name', 'recordId'].map(k => property[k]),
                       property, blockNum)
}

const addPropertyPage = (page, blockNum) => {
  return db.queryTable('properties', properties => {
    return properties
      .getAll([page.name, page.recordId], { index: 'attributes' })
      .filter({ endBlockNum: Number.MAX_SAFE_INTEGER })
  })
    .then(properties => {
      if (properties.length === 0) {
        const attrs = `${page.name}, ${page.recordId}`
        return console.warn("WARNING! Unable to find page's Property:", attrs)
      }

      const property = properties[0]

      // Convert enum indexes into names, or empty strings if not an enum
      if (property.dataType === 'ENUM') {
        page.reportedValues.forEach(reported => {
          reported.enumValue = property.enumOptions[reported.enumValue]
        })
      } else {
        page.reportedValues.forEach(reported => {
          reported.enumValue = ''
        })
      }

      // Convert `structValues` array into `structValue` object
      if (property.dataType === 'STRUCT') {
        page.reportedValues.forEach(reported => {
          reported.structValue = xformStruct(reported.structValues)
          delete reported.structValues
        })
      } else {
        page.reportedValues.forEach(reported => {
          reported.structValue = {}
          delete reported.structValues
        })
      }

    })
    .then(() => {
      return addBlockState('propertyPages', 'attributes',
                           ['name', 'recordId', 'pageNum'].map(k => page[k]),
                           page, blockNum)
    })
}

const addProposal = (proposal, blockNum) => {
  return addBlockState(
    'proposals', 'attributes',
    ['recordId', 'timestamp', 'receivingAgent', 'role'].map(k => proposal[k]),
    proposal, blockNum)
}

module.exports = {
  addAgent,
  addRecord,
  addRecordType,
  addProperty,
  addPropertyPage,
  addProposal
}
