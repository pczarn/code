import Vue from 'vue'

class robinHood {
  constructor(capacity=16, load_factor=0.9) {
    this.table = Array(capacity)
    this.size = 0
    this.load_factor = load_factor
  }

  insert(pos, value) {
    if(this.size >= this.capacity * this.load_factor) {
      this.resize(this.capacity * 2)
    }
    // remember absolute position.
    var elem = {
      text: value,
      pos: pos
    }
    // get relative position.
    pos %= this.capacity
    var elemInitial = pos
    while(this.table[pos % this.capacity] !== undefined) {
      var occupied = this.table[pos % this.capacity]
      // Bitwise, because pos - ousted.pos can be negative.
      var occupiedInitial = pos - ((pos - occupied.pos) & (this.capacity - 1))
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
    while(this.table[pos + 1] !== undefined && this.table[pos + 1].pos <= pos) {
      this.table[pos] = this.table[pos + 1]
      pos += 1
      pos %= this.capacity
    }
    // Delete.
    Vue.set(this.table, pos, undefined)
    this.size -= 1
  }

  resize(newSize) {
    var map = new robinHood(newSize, this.load_factor)
    for(var i=0; i<this.table.length; i++) {
      if(this.table[i] !== undefined) {
        map.insert(this.table[i].pos, this.table[i].text)
      }
    }
    this.table = map.table
    this.size = map.size
  }

  robinHood(pos, elem, currentInitial) {
    var ousted = this.table[pos % this.capacity]
    this.table[pos % this.capacity] = elem
    pos += 1
    while(this.table[pos % this.capacity] !== undefined) {
      var occupied = this.table[pos % this.capacity]
      var occupiedInitial = pos - ((pos - occupied.pos) & (this.capacity - 1))
      // fixme
      if(occupiedInitial > currentInitial) {
        //recurse
        this.table[pos % this.capacity] = ousted
        ousted = occupied
        currentInitial = occupiedInitial
      }
      pos += 1
    }
    Vue.set(this.table, pos % this.capacity, ousted)
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
    INSERT (state, { pos, value }) {
      // let map = toMap(state)
      state.map.insert(pos, value)
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
}
