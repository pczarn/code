"use strict";

class robinHood {
  constructor(capacity) {
    this.table = Array(capacity);
    this.size = 0;
  }

  insert(pos, elem) {
    if(this.size + 1 >= this.table.length * 0.9) {
      this.resize(this.size * 2);
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
  }

  resize(newSize) {
    var map = new robinHood(newSize);
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

  }

  draw(ctx, x=0, y=0) {

  }
}

function onLoad() {
  var canvas = document.getElementById('visualization');
  var side = 55;
  var transX = 0, transY = 0;
  var map = new robinHood(16);
  // var view = new mapView(map);

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
    ctx.fill();
  }

  function draw() {
    if(canvas.getContext) {
      var ctx = canvas.getContext('2d');
      ctx.canvas.width  = window.innerWidth;
      ctx.canvas.height  = window.innerHeight;

      ctx.globalCompositeOperation = 'destination-over';
      // clear canvas
      ctx.clearRect(0, 0, canvas.width, canvas.height);

      // drawing code
      ctx.fillStyle = "black";
      ctx.font = '12pt Calibri';
      ctx.textAlign = 'center';

      var firstEntry = map.capacity / 2 - Math.floor(canvas.width / 2 / side);
      transX = -firstEntry * side;
      transY = 30;
      ctx.translate(transX, transY);

      ctx.beginPath();
      ctx.moveTo(0, 0);
      ctx.lineTo(map.capacity * side, 0);
      ctx.moveTo(0, side);
      ctx.lineTo(map.capacity * side, side);
      // draw boxes
      var ary = [];
      var iter = map.iterator();
      // Start first square
      ctx.moveTo(0, 0);
      ctx.lineTo(0, side);
      // Squares closed
      for(var i=0; i<map.capacity; i++) {
        var next = iter.next();
        ctx.moveTo((i + 1) * side, 0);
        ctx.lineTo((i + 1) * side, side);
        if(!next.done && next.value !== undefined) {
          ctx.fillText(next.value.text, i * side + side / 2, side / 2);
          if(next.value.initial !== undefined) {
            ary.push({from: i, to: next.value.initial});
          }
        }
      }
      var arrowsIn = new Set();
      for(let edge of ary) {
        let dst_x;
        if(edge.to == edge.from) {
          dst_x = edge.to * side + side / 3;
        } else {
          dst_x = edge.to * side + side * 2 / 3;
        }
        arrowsIn.add(dst_x);
      }
      ary.sort((a, b) => a.from - a.to - (b.from - b.to));
      ary.sort((a, b) => a.from - b.from);
      var levels = [];
      while(ary.length != 0) {
        var new_ary = [];
        var current_level = [];
        for(var idx=0; idx<ary.length; idx++) {
          var next = ary[idx];
          var skip_until = next.from;
          while(idx + 1 < ary.length && ary[idx + 1].to < skip_until) {
            new_ary.push(ary[idx + 1]);
            idx += 1;
          }
          current_level.push(next);
        }
        ary = new_ary;
        levels.push(current_level);
      }
      for(let levelId=0; levelId<levels.length; levelId++) {
        for(let edge of levels[levelId]) {
          let y = side + levelId * 10;
          if(edge.to != edge.from) {
            y += 10;
          }
          let src_x = edge.from * side + side / 2;
          ctx.moveTo(src_x, side * 4 / 5);
          ctx.lineTo(src_x, y + side / 5);
          let dst_x;
          if(edge.to == edge.from) {
            dst_x = edge.to * side + side / 3;
          } else {
            dst_x = edge.to * side + side * 2 / 3;
          }
          ctx.lineTo(dst_x, y + side / 5);
          ctx.lineTo(dst_x, side * 4 / 5);
        }
      }
      ctx.stroke();

      for(let dst_x of arrowsIn) {
          ctx.beginPath();
          drawArrow(ctx, {x: dst_x, y: side * 4 / 5}, Math.PI*3/2, 7);
      }
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
    canvas.addEventListener('mouseup',function(event) {
      var pos = getMousePos(canvas, event);
      pos.y -= 15;
      if(pos.y >= 0 && pos.y < side) {
        var bucketId = Math.floor(pos.x / side);
        if(event.button == 0) {
          var text = "abc" + Math.floor(Math.random() * 100);
          map.insert(bucketId, text);
        } else {
          map.remove(bucketId);
        }
      }
    });
    window.requestAnimationFrame(draw);
  }
}

document.addEventListener('DOMContentLoaded', onLoad, false);
