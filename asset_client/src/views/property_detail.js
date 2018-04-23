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

const m = require('mithril')
const _ = require('lodash')

const api = require('../services/api')
const payloads = require('../services/payloads')
const parsing = require('../services/parsing')
const transactions = require('../services/transactions')
const layout = require('../components/layout')
const { LineGraphWidget, MapWidget } = require('../components/data')
const { Table, PagingButtons } = require('../components/tables')

const PAGE_SIZE = 50

const withIntVal = fn => m.withAttr('value', v => fn(parsing.toInt(v)))

const typedWidget = state => {
  const property = _.get(state, 'property', {})

  if (property.dataType === 'LOCATION') {
    return m(MapWidget, {
      coordinates: property.updates.map(update => update.value)
    })
  }

  if (property.dataType === 'NUMBER') {
    return m(LineGraphWidget, { updates: property.updates })
  }

  return null
}

const updateSubmitter = state => e => {
  e.preventDefault()
  const { name, dataType, recordId } = state.property

  let value = null
  if (state.update) {
    value = state.update
  } else {
    value = state.tmp
  }

  const update = { name }
  update.dataType = payloads.updateProperties.enum[dataType]
  update[`${dataType.toLowerCase()}Value`] = value

  const payload = payloads.updateProperties({
    recordId,
    properties: [update]
  })

  transactions.submit(payload, true)
    .then(() => api.get(`records/${recordId}/${name}`))
    .then(property => {
      _.each(e.target.elements, el => { el.value = null })
      state.update = null
      state.tmp = {}
      property.updates.forEach(update => {
        update.value = parsing.floatifyValue(update.value)
      })
      state.property = property
    })
}

// Produces an input field particular to the type of data
const typedInput = state => {
  if (state.property.dataType === 'NUMBER') {
    return m('.col-md-8', [
      m('input.form-control', {
        placeholder: 'Enter New Value...',
        oninput: withIntVal(value => { state.update = value })
      })
    ])
  }

  if (state.property.dataType === 'LOCATION') {
    return [
      m('.col.md-4.mr-1',
        m('input.form-control', {
          placeholder: 'Enter New Latitude...',
          oninput: withIntVal(value => { state.tmp.latitude = value })
        })),
      m('.col.md-4',
        m('input.form-control', {
          placeholder: 'Enter New Longitude...',
          oninput: withIntVal(value => { state.tmp.longitude = value })
        }))
    ]
  }

  return m('.col-md-8', [
    m('input.form-control', {
      placeholder: 'Enter New Value...',
      oninput: m.withAttr('value', value => { state.update = value })
    })
  ])
}

const updateForm = state => {
  const inputField = typedInput(state)
  if (!inputField) return null

  return m('form.my-5', {
    onsubmit: updateSubmitter(state)
  }, [
    m('.container',
      m('.row.justify-content-center',
        inputField,
        m('.col-md-2',
          m('button.btn.btn-primary', { type: 'submit' }, 'Update'))))
  ])
}

/**
 * Displays updates to a property, and form for submitting new updates.
 */
const PropertyDetailPage = {
  oninit (vnode) {
    vnode.state.currentPage = 0
    vnode.state.tmp = {}

    const refresh = () => {
      api.get(`records/${vnode.attrs.recordId}/${vnode.attrs.name}`)
        .then(property => {
          property.updates.forEach(update => {
            update.value = parsing.floatifyValue(update.value)
          })
          vnode.state.property = property
        })
        .then(() => { vnode.state.refreshId = setTimeout(refresh, 2000) })
    }

    refresh()
  },

  onbeforeremove (vnode) {
    clearTimeout(vnode.state.refreshId)
  },

  view (vnode) {
    const name = _.capitalize(vnode.attrs.name)
    const record = vnode.attrs.recordId

    const reporters = _.get(vnode.state, 'property.reporters', [])
    const isReporter = reporters.includes(api.getPublicKey())

    const updates = _.get(vnode.state, 'property.updates', [])
    const page = updates.slice(vnode.state.currentPage * PAGE_SIZE,
                               (vnode.state.currentPage + 1) * PAGE_SIZE)

    return [
      layout.title(`${name} of ${record}`),
      typedWidget(vnode.state),
      isReporter ? updateForm(vnode.state) : null,
      m('.container',
        layout.row([
          m('h5.mr-auto', 'Update History'),
          m(PagingButtons, {
            setPage: page => { vnode.state.currentPage = page },
            currentPage: vnode.state.currentPage,
            maxPage: updates.length / PAGE_SIZE
          })
        ]),
        m(Table, {
          headers: ['Value', 'Reporter', 'Time'],
          rows: page.map(update => {
            return [
              parsing.stringifyValue(update.value,
                                     vnode.state.property.dataType,
                                     vnode.state.property.name),
              update.reporter.name,
              parsing.formatTimestamp(update.timestamp)
            ]
          }),
          noRowsText: 'This property has never been updated'
        }))
    ]
  }
}

module.exports = PropertyDetailPage
