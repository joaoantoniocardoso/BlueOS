/// <reference types="vite/client" />
import './cosmos'
import '@/style/css/vuetify-global.css'
import '@/style/css/animations.css'
import 'vue-tour/dist/vue-tour.css'

import * as Sentry from '@sentry/vue'
import Vue from 'vue'
import VueApexCharts from 'vue-apexcharts'
import JsonViewer from 'vue-json-viewer'
import VueTooltipDirective from 'vue-tooltip-directive'
import VStep from 'vue-tour/src/components/VStep.vue'
import VTour from 'vue-tour/src/components/VTour.vue'
import VueDraggable from 'vuedraggable'
import Vuetify from 'vuetify/lib'

import consoleLogger from '@/libs/console-logger'
import { loadBlueosVersion } from '@/utils/blueos-version'
import { convertGitDescribeToTag } from '@/utils/helper_functions'

import App from './App.vue'
import DefaultTooltip from './components/common/DefaultTooltip.vue'
import vuetify from './plugins/vuetify'
import router from './router'
import store from './store'

Vue.use(VueTooltipDirective, {
  component: DefaultTooltip,
})
Vue.use(VueApexCharts)
Vue.use(Vuetify)
Vue.use(JsonViewer)

Vue.component('Apexchart', VueApexCharts)
Vue.component('Draggable', VueDraggable)

// Do Vue-Tour registration manually
Vue.component('VTour', VTour)
Vue.component('VStep', VStep)
Vue.prototype.$tours = {}

async function start(): Promise<void> {
  const version = await loadBlueosVersion()
  const release = `BlueOS@${version}`
  console.info(`Running: ${release}`)
  // Distance 0 is a build of the tag itself. Later commits stay out of Sentry.
  if (convertGitDescribeToTag(version)) {
    Sentry.init({
      Vue,
      release,
      dsn: 'https://d87285a04a74f71aac13445f60506708@o4507696465707008.ingest.us.sentry.io/4507765318615040',
      integrations: [
        Sentry.browserTracingIntegration({ router }),
        Sentry.replayIntegration(),
        Sentry.feedbackIntegration({ autoInject: false }),
      ],
      tracesSampleRate: 1.0,
      tracePropagationTargets: [],
      replaysSessionSampleRate: 0.1,
      replaysOnErrorSampleRate: 1.0,
      transport: Sentry.makeBrowserOfflineTransport(Sentry.makeFetchTransport),
    })
  }

  consoleLogger.initialize().catch((error) => {
    console.error('Failed to initialize console logger:', error)
  })

  new Vue({
    router,
    store,
    vuetify,
    render: (h) => h(App),
  }).$mount('#app')
}

start().catch((error) => console.error('Failed to start BlueOS:', error))
