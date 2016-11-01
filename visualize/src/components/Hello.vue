<template>
<div>
  <canvas height="600"
          width="800"
          id="visualization"
          @mouseup="mouseup"
          @mousemove="mousemove"
          @mousedown="mousedown"
          @wheel="wheel"
          @contextmenu="contextmenu"
          ref="canvas"
  >Not supported canvas</canvas>
  <div class="toolbox">
    <p>
      Click inside boxes to add elements.
      Right click to remove elements.
      Drag anywhere to move.
    </p>
    <input type="button" value="Center view" @click="resetX()">
    <input type="button" value="Reset everything" @click="resetMap()">
    <br>
    Insert random
    <template v-for="count in [1, 10, 100]">
      <template v-if="count !== 1">,</template>
      <input type="button" :value="count" @click="insertRandom(count)">
    </template>
    <br>
    <label for="load-factor">Load factor</label>
    <!-- important to have the load factor non-zero and no higher than 1 -->
    <input type="range" min="0.01" max="1" step="0.01" v-model="loadFactor">
  </div>
</div>
</template>

<script>
import { mapGetters, mapMutations, mapActions } from 'vuex'
import _ from 'lodash'

import draw from 'src/draw'

const PADDING_TOP = 70
const SIDE_LENGTH = 45

export default {
  name: "canvas-drawing",
  data () {
    return {
      loadFactor: 0.9,
      side: SIDE_LENGTH,
      transX: 0,
      transY: 0,
      transMoved: 0,
      lastX: 0,
      dragging: 0,
      highlight: false,
      clicked: null,
      msg: 'Hello Vue!',
    }
  },
  mounted () {
    window.requestAnimationFrame(this.draw)
  },
  computed: {
    edges () {
      let side = this.side
      let ary = []
      let map = this.map
      for(var i=0; i<this.capacity; i++) {
        var next = map.table[i]
        if(next !== undefined) {
          var edge = {
            from: i,
            to: next.hash % this.capacity
          }
          if(edge.to > edge.from) {
            edge.from += this.capacity
          }
          ary.push(edge)
        }
      }
      // Compute levels
      let edges = new Map()
      for(let i=0; i<ary.length; i++) {
        var to = ary[i].to
        if(!edges.has(to)) {
          edges.set(to, new Set())
        }
        edges.get(to).add(ary[i].from)
      }
      // Sort edges by key first
      edges = new Map([...edges.entries()].sort((a, b) => a[0] - b[0]))
      let processed = []
      let levels = []
      for(const [nextTo, nextFrom] of edges) {
        var skip_until = Math.max(...nextFrom.values())
        // Find a suitable level
        var freeLevel = levels.findIndex(level => level <= nextTo)
        if(freeLevel == -1) {
          freeLevel = levels.length
          levels.push(skip_until)
        } else {
          levels[freeLevel] = skip_until
        }
        // Move the edge to processed
        processed.push({
          to: nextTo,
          from: nextFrom,
          level: freeLevel
        })
      }
      return processed
    },
    highlightEdge () {
      return this.edges.find(edge => edge.to === this.highlight)
    },
    upArrows () {
      if(this.clicked) {
        const map = _.cloneDeep(this.map)
        map.moves = []
        if(this.clicked.click === 'left') {
          map.insert(this.clicked.bucket, null)
        } else {
          map.remove(this.clicked.bucket)
        }
        return map.moves
      } else {
        return []
      }
    },
    ...mapGetters(['map', 'capacity']),
  },
  methods: {
    draw () {
      const canvas = this.$refs.canvas
      if(canvas.getContext) {
        var ctx = canvas.getContext('2d')
        ctx.canvas.width = window.innerWidth
        ctx.canvas.height = window.innerHeight

        draw.setup(canvas, ctx, this.transX, this.transY)

        const firstEntry = this.capacity / 2 - Math.floor(canvas.width / 2 / this.side)
        this.transX = -firstEntry * this.side + this.transMoved
        this.transY = PADDING_TOP
        ctx.translate(this.transX, this.transY)

        const first = Math.floor(-this.transX / (this.side * this.map.capacity))
        const last = Math.ceil((-this.transX + canvas.width) / (this.side * this.map.capacity))
        for(let i=first; i<last; i++) {
          ctx.save()
          ctx.translate(i * this.map.capacity * this.side, 0)
          if(i & 1 == 1) {
            ctx.strokeStyle = "#dddddd"
            ctx.fillStyle = "#dddddd"
          }
          this.drawBuckets(ctx)
          ctx.restore()
        }
      }

      window.requestAnimationFrame(this.draw)
    },
    drawBuckets (ctx) {
      this.drawBoxes(ctx)
      for(let edge of this.edges) {
        if(edge.to != this.highlight) {
          this.drawEdgeSet(ctx, edge)
        }
      }
      ctx.stroke()
      if(this.highlightEdge !== undefined) {
        // highlighted
        ctx.save()
        ctx.strokeStyle = "red"
        ctx.fillStyle = "red"
        ctx.beginPath()
          this.drawEdgeSet(ctx, this.highlightEdge)
        ctx.stroke()
        ctx.restore()
      }
      this.drawArrows(ctx)
    },
    drawBoxes(ctx) {
      var side = this.side
      // Draw horizontal boundaries
      ctx.beginPath()
      ctx.moveTo(0, 0)
      ctx.lineTo(this.map.capacity * side, 0)
      ctx.moveTo(0, side)
      ctx.lineTo(this.map.capacity * side, side)
      // Draw boxes
      var iter = this.map.iterator()
      // Start first square
      ctx.moveTo(0, 0)
      ctx.lineTo(0, side)
      // Draw closed squares
      for(var i=0; i<this.map.capacity; i++) {
        ctx.moveTo((i + 1) * side, 0)
        ctx.lineTo((i + 1) * side, side)
      }
      ctx.stroke()
      for(var i=0; i<this.map.capacity; i++) {
        var next = iter.next()
        if(!next.done && next.value !== undefined) {
          draw.shape(ctx, next.value.value, i * side, 0, side)
        }
      }
    },
    drawEdgeSet (ctx, edgeSet) {
      ctx.beginPath()
        const side = this.side
        const y = side + edgeSet.level * 10
        let dst_x
        if(edgeSet.from.has(edgeSet.to)) {
          // Displacement of 0 present.
          dst_x = edgeSet.to * side + side / 3
        } else {
          // This must be farther to the right.
          dst_x = edgeSet.to * side + side * 2 / 3
        }
        for(let fromEntry of edgeSet.from) {
          let src_x
          if(edgeSet.to == fromEntry) {
            src_x = fromEntry * side + side * 2 / 3
          } else {
            src_x = fromEntry * side + side / 2
          }
          ctx.moveTo(src_x, side * 4 / 5)
          ctx.lineTo(src_x, y + side / 5)
          ctx.lineTo(dst_x, y + side / 5)
          ctx.lineTo(dst_x, side * 4 / 5)
        }
      ctx.stroke()
      ctx.beginPath()
        draw.arrow(ctx, {x: dst_x, y: side * 4 / 5}, Math.PI*3/2, 7)
      ctx.fill()
    },
    drawArrows (ctx) {
      const side = this.side
      ctx.beginPath()
      for(let [from, to] of this.upArrows) {
        if(from === to) {
          continue
        }
        let toArrowX, arrowAngle
        if(from) {
          let up
          if(Math.abs(from - to) === 1) {
            up = -20
            arrowAngle = Math.PI * 2 / 16
          } else if(Math.abs(from - to) < 5) {
            up = -30
            arrowAngle = Math.PI * 3 / 32
          } else if(Math.abs(from - to) < 10) {
            up = -30
            arrowAngle = Math.PI / 16
          } else {
            up = -60
            arrowAngle = Math.PI / 24
          }
          const third = side / 3
          let fromOffset, toOffset
          if(to > from) {
            fromOffset = third * 2
            toOffset = side / 2
          } else {
            fromOffset = side / 2
            toOffset = third * 2
          }
          ctx.moveTo(from * side + fromOffset, -10)
          ctx.quadraticCurveTo((from + to) / 2 * side + side / 2, up, to * side + toOffset, -10)
          toArrowX = to * side + toOffset
        } else {
          ctx.moveTo(to * side + side / 2, -30)
          ctx.lineTo(to * side + side / 2, -10)
          toArrowX = to * side + side / 2
          arrowAngle = Math.PI/2
        }
        if(to < from) {
          arrowAngle = Math.PI - arrowAngle
        }
        draw.arrow(ctx, { x: toArrowX, y: -10 }, arrowAngle)
      }
      ctx.stroke()
    },

    mousemove (e) {
      const event = e || event
      this.highlight = null
      if(this.dragging > 0) {
        const delta = event.offsetX - this.lastX
        this.transMoved += delta
        this.lastX = event.offsetX
      } else {
        const bucket = this.bucketFromMousePos(event)
        if(bucket !== null) {
          this.highlight = bucket
        }
      }
    },
    mousedown (e) {
      const event = e || event
      this.lastX = event.offsetX
      const bucket = this.bucketFromMousePos(event)
      if(bucket === null) {
        // dragging outside the box
        this.dragging += 1
      } else {
        if(event.button === 0) {
          this.clicked = { bucket, click: 'left' }
        } else if(event.button === 2) {
          this.clicked = { bucket, click: 'right' }
        }
      }
      event.stopPropagation()
    },
    mouseup (event) {
      this.clicked = null
      if(this.dragging !== 0) {
        this.dragging -= 1
      }
      const bucket = this.bucketFromMousePos(event)
      if(bucket !== null) {
        event.stopPropagation()
        if(event.button == 0) {
          this.insert(bucket)
        } else {
          this.remove(bucket)
        }
      }
    },
    wheel (event) {
      if(event.deltaMode == MouseEvent.DOM_DELTA_PIXEL) {
        this.transMoved += event.deltaY
      } else {
        this.transMoved += event.deltaY * 35
      }
    },
    contextmenu (event) {
      var bucket = this.bucketFromMousePos(event)
      if(bucket) {
        event.preventDefault()
        return false
      }
    },

    getMousePos (event) {
      var rect = this.$refs.canvas.getBoundingClientRect()
      return {
        x: event.clientX - rect.left - this.transX,
        y: event.clientY - rect.top - this.transY,
      }
    },
    bucketFromMousePos (event) {
      var pos = this.getMousePos(event)
      if(pos.y >= 0 && pos.y <= this.side && pos.x >= 0 && pos.x <= this.map.capacity * this.side) {
        return Math.floor(pos.x / this.side)
      } else {
        return null
      }
    },

    resetMap (event) {
      this.reset()
      this.transMoved = 0
    },
    resetX (event) {
      this.transMoved = 0
    },
    insertRandom (count) {
      for(var i=0; i<count; i++) {
        var randomBucket = Math.floor(Math.random() * this.capacity)
        this.insert(randomBucket)
      }
    },
    ...mapMutations({
      setCapacity: 'SET_CAPACITY',
      // insert: 'INSERT',
      remove: 'REMOVE',
      resize: 'RESIZE',
      reset: 'RESET',
    }),
    ...mapActions({
      insert: 'INSERT_RANDOM',
    })
  }
}
</script>

<style scoped>
canvas {
  position: absolute;
}

.toolbox {
  position: fixed;
  bottom: 50px;
  left: 50px;
  border: 2px solid gray;
  padding: 12px 12px;
}
</style>
