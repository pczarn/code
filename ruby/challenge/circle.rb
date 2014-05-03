#!/usr/bin/env ruby1.9.1

r = ARGV[1].to_i
thickness = ARGV[2].to_f
char_width = 10.0
char_height = 19.0
chars = (0x21 .. 0x7e).to_a #+ (0x2580 .. 0x25ff).to_a
x_scale = 1 #char_width / char_height
y_scale = char_height / char_width  #1
range = (r - thickness .. r + thickness)
best_full = nil

h = Hash[ chars.map {|c|
   [c, IO.popen("convert -size #{char_width}x#{char_height} -depth 8 -font '#{ARGV[0]}' -background black -fill white -pointsize 16 label:@- gray:-", 'r+') {|io|
      io.write [c].pack("U")
      io.close_write
      io.read.unpack('C*')
   }]
} ]

(-r-thickness.ceil .. r+thickness.ceil).step(y_scale) {|y|
   puts (-r-thickness.ceil .. r+thickness.ceil).step(x_scale).map {|x|
      z = Complex(x, y).abs
      next 0x20 if (z - r).abs > 1 + thickness

      a = (0 ... 1).step(1 / char_height).flat_map {|y2|
         (0 ... 1).step(1 / char_width).map {|x2|
            range.include? Complex(x + x2, y + y2).abs
         }
      }

      next best_full if best_full and !a.include? false

      best = h.map {|k, img|
         [img.each.with_index.inject(0) {|sum, (v, i)|
            sum + (a[i] ? v : -v)
         }, k]
      }.max

      if best[0] > 0
         best_full = best[1] if !a.include? false
         next best[1]
      end
      0x20
   }.pack("U*")
}
