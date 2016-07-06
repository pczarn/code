"use strict";

function drawArrow(ctx, pt, angle, size=10) {
  var alx = pt.x - size*Math.cos(angle - Math.PI/6),
      aly = pt.y - size*Math.sin(angle - Math.PI/6);
  var arx = pt.x - size*Math.cos(angle + Math.PI/6),
      ary = pt.y - size*Math.sin(angle + Math.PI/6);
  var asx = pt.x - size*.7*Math.cos(angle),
      asy = pt.y - size*.7*Math.sin(angle);
  ctx.moveTo(pt.x, pt.y);
  ctx.lineTo(alx, aly);
  ctx.lineTo(asx, asy);
  ctx.lineTo(arx, ary);
  ctx.lineTo(pt.x, pt.y);
}

class robinHood {
  constructor(capacity=16, load_factor=0.9) {
    this.table = Array(capacity);
    this.size = 0;
    this.load_factor = load_factor;
  }

  insert(pos, elem) {
    if(this.size >= this.capacity * this.load_factor) {
      this.resize(this.capacity * 2);
    }
    // robin hood here
    var initial = pos;
    while(this.table[pos] !== undefined) {
      var occupied = this.table[pos];
      // check if the occupied entry is more fortunate
      if(occupied.initial > initial) {
        // Begin robin hood
        var ousted = occupied;
        this.table[pos] = {text: elem, initial: initial};
        pos += 1;
        pos %= this.capacity;
        while(this.table[pos] !== undefined) {
          var occupied = this.table[pos];
          if(occupied.initial > ousted.initial) {
            //recurse
            this.table[pos] = ousted;
            ousted = occupied;
          }
          pos += 1;
          pos %= this.capacity;
        }
        this.table[pos] = ousted;
        this.size += 1;
        // End robin hood
        return;
      }
      pos += 1;
      pos %= this.capacity;
    }
    this.table[pos] = {text: elem, initial: initial};
    this.size += 1;
  }

  remove(pos) {
    // Back shift.
    while(this.table[pos + 1] !== undefined && this.table[pos + 1].initial <= pos) {
      this.table[pos] = this.table[pos + 1];
      pos += 1;
      pos %= this.capacity;
    }
    // Delete.
    this.table[pos] = undefined;
    this.size -= 1;
  }

  resize(newSize) {
    var map = new robinHood(newSize, this.load_factor);
    for(var i=0; i<this.table.length; i++) {
      if(this.table[i] !== undefined) {
        map.insert(this.table[i].initial, this.table[i].text);
      }
    }
    this.table = map.table;
    this.size = map.size;
  }

  robin_hood(pos) {
    // TODO
  }

  iterator() {
    return this.table[Symbol.iterator]();
  }

  get capacity() {
    return this.table.length;
  }
}

class mapView {
  constructor(map) {
    this.map = map;
    this.update();
    this.highlight = null;
  }

  update() {
    var side = this.side;
    var ary = [];
    for(var i=0; i<this.map.capacity; i++) {
      var next = this.map.table[i];
      if(next !== undefined) {
        // .initial can be undefined??
        var edge = {from: i, to: next.initial};
        if(edge.to > edge.from) {
          edge.from += this.map.capacity;
        }
        ary.push(edge);
      }
    }
    // Compute levels
    var edges = new Map();
    for(var i=0; i<ary.length; i++) {
      var to = ary[i].to;
      if(!edges.has(to)) {
        edges.set(to, new Set());
      }
      edges.get(to).add(ary[i].from);
    }
    var processed = [];
    var levels = [];
    for(var [nextTo, nextFrom] of edges) {
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
    // Save work
    this.edges = processed;
  }

  draw(ctx) {
    var side = this.side;
    this.drawBoxes(ctx);
    for(let edge of this.edges) {
      if(edge.to != this.highlight) {
        this.drawEdgeSet(ctx, edge);
      }
    }
    ctx.stroke();
    var highlight = this.edges.find(edge => edge.to == this.highlight);
    if(highlight !== undefined) {
      // highlighted
      ctx.save();
      ctx.strokeStyle = "red";
      ctx.fillStyle = "red";
      ctx.beginPath();
        this.drawEdgeSet(ctx, highlight);
      ctx.stroke();
      ctx.restore();
    }
  }

  drawBoxes(ctx) {
    var side = this.side;
    // Draw horizontal boundaries
    ctx.beginPath();
    ctx.moveTo(0, 0);
    ctx.lineTo(this.map.capacity * side, 0);
    ctx.moveTo(0, side);
    ctx.lineTo(this.map.capacity * side, side);
    // Draw boxes
    var iter = this.map.iterator();
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
  }

  drawEdgeSet(ctx, edgeSet) {
    ctx.beginPath();
      var side = this.side;
      let y = side + edgeSet.level * 10;
      let dst_x;
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
  }
}

function onLoad() {
  var canvas = document.getElementById('visualization');
  var transX = 0, transY = 0, transMoved = 0;
  var map = new robinHood();
  var view = new mapView(map);
  view.side = 55;

  function draw() {
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
      transY = 30;
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
          map.insert(bucket, text);
        } else {
          map.remove(bucket);
        }
        view.update();
      }
    });
    window.requestAnimationFrame(draw);
  }

  var reset_btn = document.getElementById('reset-map');

  reset_btn.addEventListener('click', function(event) {
    map = new robinHood();
    view.map = map;
    view.update();
  });

  var reset_pos_btn = document.getElementById('reset-x');

  reset_pos_btn.addEventListener('click', function(event) {
    transMoved = 0;
  });

  var insert_random = document.getElementById('insert-random');

  insert_random.addEventListener('click', function(event) {
    for(var i=0; i<10; i++) {
      var text = "el" + Math.floor(Math.random() * 100);
      map.insert(Math.floor(Math.random() * map.capacity), text);
    }
    view.update();
  });
}

document.addEventListener('DOMContentLoaded', onLoad, false);
