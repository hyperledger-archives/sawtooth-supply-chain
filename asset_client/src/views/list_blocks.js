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
const truncate = require('lodash/truncate')
const {Table, FilterGroup, PagingButtons} = require('../components/tables')
const api = require('../services/api')
const { formatTimestamp } = require('../services/parsing')

const PAGE_SIZE = 50

let dialog = {};
dialog.view = function(ctrl, opts) {
    var batches = opts.batches.map(function (x) { return m('li', x.header_signature)});
    var transactions = opts.batches.reduce(function (x, y) { return x.concat(y.transactions.map(function(z){ return m('li', z.payload + " (sha512: " + z.header.payload_sha512 + ")");}))},[]) ;

    var content = m('.dialogContent', [
      m('h2', opts.rec.header_signature),
      m('div', [
          m('div', 'Previous block'),
          m('div', opts.rec.previous_block_id)]),
      m('h3', 'Batches'),
      m('ul', [
          batches
      ]),
      m('h3', 'Transactions'),
      m('ul', [
          transactions
      ]),
      m('a', {
          href:'#',
          onclick: closeDialog
      }, 'Close')
    ])
    return m('.dialog', content);
};

let closeDialog = function(e) {
    m.mount(dialogContainer, null);  
};

let openDialog = function(e, opts) {
    let dialogContainer = document.getElementById('dialogContainer');
    if (!dialogContainer) {
        dialogContainer = document.createElement('div');
        dialogContainer.id = 'dialogContainer';
        document.body.appendChild(dialogContainer);
    }
    m.mount(dialogContainer, m(dialog, opts));
};


const AssetList = {
  oninit (vnode) {
    vnode.state.records = []
    vnode.state.filteredRecords = []

    vnode.state.currentPage = 0

    const refresh = () => {
        api.get('blocks').then((records) => {
            vnode.state.records = records.data
            //vnode.state.records.sort((a, b) => {
            //  return getLatestPropertyUpdateTime(b) - getLatestPropertyUpdateTime(a)
            //})
            vnode.state.filteredRecords = vnode.state.records
          })
          .then(() => { vnode.state.refreshId = setTimeout(refresh, 2000) })
    }

    refresh()
  },

  onbeforeremove (vnode) {
    clearTimeout(vnode.state.refreshId)
  },

  view (vnode) {
    let publicKey = api.getPublicKey()
    return [
      m('.asset-table',
        m('.row.btn-row.mb-2', _controlButtons(vnode, publicKey)),
        m(Table, {
          headers: [
            'Block #',
            'Block ID',
            'Batches',
            'Transactions',
            'Signer'
          ],
          rows: vnode.state.filteredRecords.slice(
            vnode.state.currentPage * PAGE_SIZE,
            (vnode.state.currentPage + 1) * PAGE_SIZE)
                .map((rec) => [
                  rec.header.block_num,
                  truncate(rec.header_signature, { length: 64 }),
                  rec.batches.length,
                  rec.batches.reduce(function(accumulator, batch) { return accumulator + batch.transactions.length}, 0),
                  truncate(rec.header.signer_public_key, { length: 32 })
                ]),
          noRowsText: 'No records found'
        })
      )
    ]
  }
}

const _controlButtons = (vnode, publicKey) => {
  if (publicKey) {
    let filterRecords = (f) => {
      vnode.state.filteredRecords = vnode.state.records.filter(f)
    }

    return [
      m('.col-sm-8',
        m(FilterGroup, {
          ariaLabel: 'Filter Based on Ownership',
          filters: {
            'All': () => { vnode.state.filteredRecords = vnode.state.records }
          },
          initialFilter: 'All'
        })),
      m('.col-sm-4', _pagingButtons(vnode))
    ]
  } else {
    return [
      m('.col-sm-4.ml-auto', _pagingButtons(vnode))
    ]
  }
}

const _pagingButtons = (vnode) =>
  m(PagingButtons, {
    setPage: (page) => { vnode.state.currentPage = page },
    currentPage: vnode.state.currentPage,
    maxPage: Math.floor(vnode.state.filteredRecords.length / PAGE_SIZE)
  })

module.exports = AssetList
