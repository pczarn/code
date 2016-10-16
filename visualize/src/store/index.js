import Vue from 'vue'
import Vuex from 'vuex'
import { robinHoodModule } from './robin_hood'

Vue.use(Vuex)

export const store = new Vuex.Store({
  modules: {
    map: robinHoodModule
  }
})

// export const store = new Vuex.Store(robinHoodModule)
