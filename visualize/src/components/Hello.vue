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
      <input type="button" value="Insert 10 random" @click="insertRandom(10)"><br>
      <label for="load-factor">Load factor</label>
      <!-- important to have the load factor non-zero and no higher than 1 -->
      <input type="range" min="0.01" max="1" step="0.01" v-model="loadFactor">
    </div>
  </div>
</template>

<script>
import { mapGetters, mapMutations } from 'vuex'
import { drawArrow } from 'src/draw'

const PADDING_TOP = 60;
const SIDE_LENGTH = 45;

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
      dragging: false,
      highlight: false,
      msg: 'Hello Vue!',
    }
  },
  mounted () {
    window.requestAnimationFrame(this.draw);
  },
  computed: {
    edges () {
      let side = this.side;
      let ary = [];
      let map = this.map;
      for(var i=0; i<this.capacity; i++) {
        var next = map.table[i];
        if(next !== undefined) {
          var edge = {
            from: i,
            to: next.pos % this.capacity
          };
          if(edge.to > edge.from) {
            edge.from += this.capacity;
          }
          ary.push(edge);
        }
      }
      // Compute levels
      let edges = new Map();
      for(let i=0; i<ary.length; i++) {
        var to = ary[i].to;
        if(!edges.has(to)) {
          edges.set(to, new Set());
        }
        edges.get(to).add(ary[i].from);
      }
      // Sort edges by key first
      edges = new Map([...edges.entries()].sort((a, b) => a[0] - b[0]));
      let processed = [];
      let levels = [];
      for(const [nextTo, nextFrom] of edges) {
        var skip_until = Math.max(...nextFrom.values());
        // Find a suitable level
        var freeLevel = levels.findIndex(level => level <= nextTo);
        if(freeLevel == -1) {
          freeLevel = levels.length;
          levels.push(skip_until);
        } else {
          levels[freeLevel] = skip_until;
        }
        // Move the edge to processed
        processed.push({
          to: nextTo,
          from: nextFrom,
          level: freeLevel
        });
      }
      return processed
    },
    ...mapGetters(['map', 'capacity']),
  },
  methods: {
    draw () {
      const canvas = this.$refs.canvas
      if(canvas.getContext) {
        var ctx = canvas.getContext('2d');
        ctx.canvas.width  = window.innerWidth;
        ctx.canvas.height  = window.innerHeight;

        ctx.globalCompositeOperation = 'destination-over';
        // clear canvas
        ctx.clearRect(-this.transX, -this.transY, canvas.width, canvas.height);

        // drawing code
        ctx.strokeStyle = "black";
        ctx.fillStyle = "black";
        ctx.font = '12pt Calibri';
        ctx.textAlign = 'center';

        const firstEntry = this.capacity / 2 - Math.floor(canvas.width / 2 / this.side);
        this.transX = -firstEntry * this.side + this.transMoved;
        this.transY = PADDING_TOP;
        ctx.translate(this.transX, this.transY);
        this.drawBuckets(ctx)

        ctx.save();
        ctx.translate(-this.map.capacity * this.side, 0);
        ctx.strokeStyle = "#cccccc";
        ctx.fillStyle = "#cccccc";
        this.drawBuckets(ctx)

        ctx.restore();
        ctx.translate(this.map.capacity * this.side, 0);
        ctx.strokeStyle = "#cccccc";
        ctx.fillStyle = "#cccccc";
        this.drawBuckets(ctx)
      }

      window.requestAnimationFrame(this.draw);
    },
    drawBuckets (ctx) {
      this.drawBoxes(ctx);
      for(let edge of this.edges) {
        if(edge.to != this.highlight) {
          this.drawEdgeSet(ctx, edge);
        }
      }
      ctx.stroke();
      var highlightEdge = this.edges.find(edge => edge.to === this.highlight)
      if(highlightEdge !== undefined) {
        // highlighted
        ctx.save();
        ctx.strokeStyle = "red";
        ctx.fillStyle = "red";
        ctx.beginPath();
          this.drawEdgeSet(ctx, highlightEdge);
        ctx.stroke();
        ctx.restore();
      }
    },
    drawBoxes(ctx) {
      var side = this.side;
      // Draw horizontal boundaries
      ctx.beginPath();
      ctx.moveTo(0, 0);
      ctx.lineTo(this.map.capacity * side, 0);
      ctx.moveTo(0, side);
      ctx.lineTo(this.map.capacity * side, side);
      // Draw boxes
      var iter = this.map.iterator()
      // Start first square
      ctx.moveTo(0, 0);
      ctx.lineTo(0, side);
      // Draw closed squares
      for(var i=0; i<this.map.capacity; i++) {
        var next = iter.next();
        ctx.moveTo((i + 1) * side, 0);
        ctx.lineTo((i + 1) * side, side);
        if(!next.done && next.value !== undefined) {
          ctx.fillText(next.value.text, i * side + side / 2, side / 2);
        }
      }
      ctx.stroke();
    },
    drawEdgeSet (ctx, edgeSet) {
      ctx.beginPath();
        const side = this.side;
        const y = side + edgeSet.level * 10;
        let dst_x
        if(edgeSet.from.has(edgeSet.to)) {
          // Displacement of 0 present.
          dst_x = edgeSet.to * side + side / 3;
        } else {
          // This must be farther to the right.
          dst_x = edgeSet.to * side + side * 2 / 3;
        }
        for(let fromEntry of edgeSet.from) {
          let src_x;
          if(edgeSet.to == fromEntry) {
            src_x = fromEntry * side + side * 2 / 3;
          } else {
            src_x = fromEntry * side + side / 2;
          }
          ctx.moveTo(src_x, side * 4 / 5);
          ctx.lineTo(src_x, y + side / 5);
          ctx.lineTo(dst_x, y + side / 5);
          ctx.lineTo(dst_x, side * 4 / 5);
        }
      ctx.stroke();
      ctx.beginPath();
        drawArrow(ctx, {x: dst_x, y: side * 4 / 5}, Math.PI*3/2, 7);
      ctx.fill();
    },

    mousemove (e) {
      const event = e || event;
      this.highlight = null;
      if(this.dragging) {
        const delta = event.offsetX - this.lastX;
        this.transMoved += delta;
        this.lastX = event.offsetX;
      } else {
        const bucket = this.bucketFromMousePos(event)
        if(bucket !== null) {
          this.highlight = bucket;
        }
      }
    },
    mousedown (e) {
      const event = e || event;
      this.lastX = event.offsetX;
      const bucket = this.bucketFromMousePos(event)
      if(bucket === null) {
        // dragging outside the box
        this.dragging = true;
      }
    },
    mouseup (event) {
      this.dragging = false;
      const bucket = this.bucketFromMousePos(event)
      if(bucket !== null) {
        if(event.button == 0) {
          var text = "el" + Math.floor(Math.random() * 100);
          var randomInt = Math.floor(Math.random() * (1 << 16));
          this.insert({
            pos: bucket + randomInt * this.capacity,
            value: text
          });
        } else {
          this.remove(bucket);
        }
      }
    },
    wheel (event) {
      if(event.deltaMode == MouseEvent.DOM_DELTA_PIXEL) {
        this.transMoved += event.deltaY;
      } else {
        this.transMoved += event.deltaY * 35;
      }
    },
    contextmenu (event) {
      var bucket = this.bucketFromMousePos(event)
      if(bucket) {
        event.preventDefault();
        return false;
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
      this.transMoved = 0;
    },
    resetX (event) {
      this.transMoved = 0;
    },
    insertRandom (event) {
      for(var i=0; i<10; i++) {
        var text = "el" + Math.floor(Math.random() * 100);
        var randomInt = Math.floor(Math.random() * (1 << 16));
        this.insert({ pos: randomInt, value: text });
      }
    },
    ...mapMutations({
      setCapacity: 'SET_CAPACITY',
      insert: 'INSERT',
      remove: 'REMOVE',
      resize: 'RESIZE',
      reset: 'RESET',
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
