"use strict"
import Vue from 'vue'

export const TRIANGLE = 0
export const RECTANGLE = 1
export const CIRCLE = 2
export const NUM_SHAPES = CIRCLE + 1

class robinHood {
  constructor(capacity=16, load_factor=0.9) {
    this.table = Array(capacity)
    this.size = 0
    this.load_factor = load_factor
  }

  insert(hash, value) {
    if(this.size >= this.capacity * this.load_factor) {
      this.resize(this.capacity * 2)
    }
    // remember absolute position.
    var elem = {
      value: value,
      hash: hash,
    }
    // get relative position.
    let pos = hash % this.capacity
    const elemInitial = pos
    while(this.table[pos % this.capacity] !== undefined) {
      var occupied = this.table[pos % this.capacity]
      // Bitwise, because pos - ousted.pos can be negative.
      var occupiedInitial = pos - ((pos - occupied.hash) & (this.capacity - 1))
      // check if the occupied entry is more fortunate
      if(occupiedInitial > elemInitial) {
        // Begin robin hood
        this.robinHood(pos, elem, occupiedInitial)
        return
      }
      pos += 1
      // Sanity assert
      if(pos >= elemInitial + this.size + 1) {
        // error
        return
      }
    }
    Vue.set(this.table, pos % this.capacity, elem)
    this.size += 1
  }

  remove(pos) {
    // Back shift.
    while(this.table[pos + 1] !== undefined && this.table[pos + 1].hash % this.capacity <= pos) {
      this.table[pos] = this.table[pos + 1]
      pos += 1
      pos %= this.capacity
    }
    // Delete.
    if(this.table[pos] !== undefined) {
      Vue.set(this.table, pos, undefined)
      this.size -= 1
    }
  }

  resize(newSize) {
    var map = new robinHood(newSize, this.load_factor)
    for(var i=0; i<this.table.length; i++) {
      if(this.table[i] !== undefined) {
        map.insert(this.table[i].hash, this.table[i].value)
      }
    }
    this.table = map.table
    this.size = map.size
  }

  robinHood(pos, unbound, currentInitial)
    while(this.table[pos % this.capacity] !== undefined) {
      // forward-shift
      const tmpElem = this.table[pos % this.capacity]
      this.table[pos % this.capacity] = unbound
      unbound = tmpElem
      pos += 1
    }
    Vue.set(this.table, pos % this.capacity, unbound)
    this.size += 1
  }

  iterator() {
    return this.table[Symbol.iterator]()
  }

  get capacity() {
    return this.table.length
  }

  set capacity(cap) {
    this.table = Array(cap)
  }
}

// function toMap(state) {
//   let map = new RobinHood(0, state.loadFactor)
//   map.table = state.table
//   map.size = state.size
// }
//
// function fromMap(state, map) {
//   state.size = map.size
//   state.loadFactor = map.load_factor
//   state.table = map.table
// }

export const robinHoodModule = {
  state: {
    map: new robinHood(),
  },
  mutations: {
    SET_CAPACITY (state, cap) {
      state.map.capacity = cap
    },
    RESIZE (state, newSize) {
      // var map = new Vuex.Store(robinHoodModule)
      // map.state.size = newSize
      // map.state.loadFactor = state.loadFactor
      // for(var i=0; i<state.table.length; i++) {
      //   if(state.table[i] !== undefined) {
      //     map.commit('INSERT', [this.table[i].pos, this.table[i].text])
      //   }
      // }
      // state.table = map.state.table
      // state.size = map.state.
      // let map = toMap(state)
      state.map.resize(newSize)
      // fromMap(state, pos)
    },
    INSERT (state, { hash, value }) {
      // let map = toMap(state)
      state.map.insert(hash, value)
      // fromMap(state, pos)
    },
    REMOVE (state, pos) {
      // let map = toMap(state)
      state.map.remove(pos)
      // fromMap(state, pos)
    },
    RESET (state) {
      state.map = new robinHood()
    },
  },
  getters: {
    capacity (state) {
      return state.map.capacity
    },
    map (state) {
      return state.map
    },
    iterator (state) {
      return state.map.iterator()
    },
  },
  actions: {
    INSERT_RANDOM ({ getters, commit }, bucket) {
      var randomInt = Math.floor(Math.random() * (1 << 16))
      const r = _ => Math.floor(Math.random() * 256)
      const c = _ => `rgb(${r()}, ${r()}, ${r()})`
      const value = {
        colors: [c(), c()],
        angle: Math.random() * Math.PI,
        shape: Math.floor(Math.random() * NUM_SHAPES),
      }
      commit('INSERT', {
        hash: bucket + randomInt * getters.capacity,
        value: value,
      })
    },
  },
}
