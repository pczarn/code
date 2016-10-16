<template>
  <div>
    test2
    <canvas height="600" width="800" id="visualization">Not supported canvas</canvas>
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
export default {
  name: "canvas-drawing",
  data () {
    return {
      loadFactor: 0.9,
      msg: 'Hello Vue!'
    }
  },
  // methods: {
    beforeUpdate () {
      console.log(123);
      var canvas = document.getElementById('visualization');
      var transX = 0, transY = 0, transMoved = 0;
      view.side = SIDE_LENGTH;

      function getMousePos(canvas, event) {
        var rect = canvas.getBoundingClientRect();
        return {
          x: event.clientX - rect.left - transX,
          y: event.clientY - rect.top - transY
        };
      }

      if(canvas.getContext) {
        var lastX = 0, dragging = false;

        function bucketFromMousePos(event) {
          var pos = getMousePos(canvas, event);
          if(pos.y >= 0 && pos.y <= view.side && pos.x >= 0 && pos.x <= map.capacity * view.side) {
            return Math.floor(pos.x / view.side);
          } else {
            return null;
          }
        }

        canvas.addEventListener('mousemove', function(e) {
          var event = e || event;
          view.highlight = null;
          if(dragging) {
            var delta = event.offsetX - lastX;
            transMoved += delta;
            lastX = event.offsetX;
          } else {
            var bucket = bucketFromMousePos(event);
            if(bucket !== null) {
              view.highlight = bucket;
            }
          }
        });

        canvas.addEventListener('mousedown', function(e) {
          var event = e || event;
          lastX = event.offsetX;
          var bucket = bucketFromMousePos(event);
          if(bucket === null) {
            // dragging outside the box
            dragging = true;
          }
        });

        canvas.addEventListener('mouseup', function(event) {
          dragging = false;
          var bucket = bucketFromMousePos(event);
          if(bucket !== null) {
            var load_factor = document.getElementById('load-factor');
            map.load_factor = parseFloat(load_factor.value);
            if(event.button == 0) {
              var text = "el" + Math.floor(Math.random() * 100);
              var randomInt = Math.floor(Math.random() * (1 << 16));
              map.insert(bucket + randomInt * map.capacity, text);
            } else {
              map.remove(bucket);
            }
            view.update();
          }
        });

        canvas.addEventListener('wheel', function(event) {
          if(event.deltaMode == MouseEvent.DOM_DELTA_PIXEL) {
            transMoved += event.deltaY;
          } else {
            transMoved += event.deltaY * 35;
          }
        }, false);

        canvas.addEventListener('contextmenu', function(event) {
          var bucket = bucketFromMousePos(event);
          if(bucket) {
            event.preventDefault();
            return false;
          }
        }, false);

        window.requestAnimationFrame(draw);
      }

      var reset_btn = document.getElementById('reset-map');

      reset_btn.addEventListener('click', function(event) {
        map = new robinHood();
        view.map = map;
        view.update();
        transMoved = 0;
      });

      var reset_pos_btn = document.getElementById('reset-x');

      reset_pos_btn.addEventListener('click', function(event) {
        transMoved = 0;
      });

      var insert_random = document.getElementById('insert-random');

      insert_random.addEventListener('click', function(event) {
        for(var i=0; i<10; i++) {
          var text = "el" + Math.floor(Math.random() * 100);
          var randomInt = Math.floor(Math.random() * (1 << 16));
          map.insert(randomInt, text);
        }
        view.update();
      });
    },
  methods: {
    draw () {
      if(canvas.getContext) {
        var ctx = canvas.getContext('2d');
        ctx.canvas.width  = window.innerWidth;
        ctx.canvas.height  = window.innerHeight;

        ctx.globalCompositeOperation = 'destination-over';
        // clear canvas
        ctx.clearRect(-transX, -transY, canvas.width, canvas.height);

        // drawing code
        ctx.strokeStyle = "black";
        ctx.fillStyle = "black";
        ctx.font = '12pt Calibri';
        ctx.textAlign = 'center';

        var firstEntry = map.capacity / 2 - Math.floor(canvas.width / 2 / view.side);
        transX = -firstEntry * view.side + transMoved;
        transY = PADDING_TOP;
        ctx.translate(transX, transY);
        view.draw(ctx);

        ctx.save();
        ctx.translate(-map.capacity * view.side, 0);
        ctx.strokeStyle = "#cccccc";
        ctx.fillStyle = "#cccccc";
        view.draw(ctx);

        ctx.restore();
        ctx.translate(map.capacity * view.side, 0);
        ctx.strokeStyle = "#cccccc";
        ctx.fillStyle = "#cccccc";
        view.draw(ctx);
      }

      window.requestAnimationFrame(draw);
    }
  }
}
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
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
