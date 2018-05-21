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

const _ = require('lodash')
const { NotFound } = require('./errors')
const db = require('../db/record_types')

const FILTER_KEYS = ['name']

const fetch = ({ typeName }) => {
  return db.fetch(typeName)
    .then(resourceType => {
      if (!resourceType) {
        throw new NotFound(`No resource type with name: ${typeName}`)
      }
      return resourceType
    })
}

const list = params => db.list(_.pick(params, FILTER_KEYS))

module.exports = {
  fetch,
  list
}
