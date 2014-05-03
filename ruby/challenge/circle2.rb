#!/usr/bin/env ruby1.9.1

h = {}
a = ("\x21" .. "\x7e").map {|e| [e, e] }
a[0x3b][1] = '\\\\'
a[6][1] = %{'"'"'}
a.each {|c1, c2|
   #puts "convert -font '/usr/share/fonts/truetype/ttf-droid/DroidSansMono.ttf' -background black -fill white -pointsize 16 label:'#{c}' -write info:- gray:-"
   h[c1] = IO.popen("convert -font '/usr/share/fonts/truetype/ttf-droid/DroidSansMono.ttf' -background black -fill white -pointsize 16 label:'#{c2}' -write info:- gray:-") {|io| [io.gets.match(/LABEL ([0-9]+)x([0-9]+)/)[1, 2].map(&:to_i), io.read.unpack('C*')] }
}

xp = 1.0 / 12
yp = 1.0 / 20 

r = gets.to_i
(-r ... r).each {|y|
   (-r ... r).each {|x|
      z = Complex(x, y).abs
      res = ' '

      if (z - r).abs < 1
         res = h.map {|k, v|
            width = v.first[0]
            height = v.first[1]
            diff = 0
            v[1].each_with_index {|v, i|
               compl = Complex(x + i%width * (1.0 / width), y + i/height * (1.0 / height)).abs
               #puts "\t\t Complex(#{x} + #{i}%#{width} * (1.0 / #{width}), #{y} + #{i}/#{width} * (1.0 / #{height})).abs = Complex(#{x} + #{i%width * (1.0 / width)}, #{y} + #{i/width * (1.0 / height)}).abs = #{compl}; #{compl-1}"
               diff += (compl - r).abs < 0.2 ? v : -v
            }
            #puts "\t'#{k}': #{diff}" if diff != 0
            [k, diff]
         }.sort {|a,b| b[1]<=>a[1]}.first[0]
      end
      print res
      #puts "#{z}: #{res}"
   }
   puts# '-----------'
}
