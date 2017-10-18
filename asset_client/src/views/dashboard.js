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

const Dashboard = {
  view (vnode) {
    return [
      m('.header.text-center.mb-4',
        m('h4', 'Welcome To'),
        m('h1.mb-3', 'AssetTrack'),
        m('h5',
          m('em',
            'Powered by ',
            m('strong', 'Sawtooth Supply Chain')))),
      m('.blurb',
        m('p',
          m('em', 'Sawtooth Supply Chain'),
          ' is a general purpose supply chain solution built using the ',
          'power of ',
          m('a[href="https://github.com/hyperledger/sawtooth-core"]',
            { target: '_blank' },
            "Hyperledger Sawtooth's"),
          ' blockchain technology. It maintains a distributed ledger ',
          'that tracks both asset provenance and a timestamped history ',
          'detailing how an asset was stored, handled, and transported.'),
        m('p',
          m('em', 'AssetTrack'),
          ' demonstrates this unique technology in a simple web app. ',
          'It allows generic assets to be added to the blockchain and ',
          'tracked through their life cycle. As ownership changes hands, ',
          'location changes, or even if there is a spike in temperature ',
          'during storage, an update can be submitted to the distributed ',
          'ledger, providing an immutable auditable history.'),
        m('p',
          'To use ',
          m('em', 'AssetTrack'),
          ', create an account using the link in the navbar above. ',
          'Once logged in, you will be able to add new assets to ',
          'the blockchain and track them with data like temperature or ',
          'location. You will be able to authorize other "agents" on the ',
          'blockchain to track this data as well, or even transfer ',
          'ownership or possession of the asset entirely. For the ',
          'adventurous, these actions can also be accomplished directly ',
          'with the REST API running on the ',
          m('em', 'Supply Chain'),
          ' server, perfect for automated IoT sensors.'))
    ]
  }
}

module.exports = Dashboard
