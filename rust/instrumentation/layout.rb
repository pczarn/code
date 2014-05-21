require 'pp'

s=File.read('layout_info.txt')
cur = nil
libs = s.split(/(?=oxidize)/).map do |crate|
   lib = crate[/\/(\w+)$/, 1]
   structs = crate.scan(/has size (\d+): \[(.*)\]/).map do |sz, fsz|
      [sz.to_i, fsz.split(', ').map(&:to_i)]
   end
   [lib, structs]
end

pp libs
exit

dane = libs.map do |((lib, _, _)), stats|
   [
      lib,
      stats && stats.map {|_, sz, fsz| [sz.to_i, *(fsz ? fsz.split.map(&:to_i) : [])] }
   ]
end

sz_csv = {}
al_csv = {}
diff_csv = {}
sz_csv.default = al_csv.default = diff_csv.default = 0

p dane.map{|lib,_|lib}

dane.map {|lib, c|
   c.map {|st| [st.first, st.drop(1).inject(&:+) || 0] } if c
}.flatten(1).each {|sz, al|
   sz_csv[sz] += 1 if sz && sz > 0
   al_csv[al] += 1 if al && al > 0
   diff_csv[sz - al] += 1 if sz && al
}

puts diff_csv.flat_map{|a|a ? a.join(' | ') : []}.join("\n")

File.write('layout_sz.csv', sz_csv.flat_map{|a|a ? a.join(',') : []}.join("\n"))
File.write('layout_al.csv', al_csv.flat_map{|a|a ? a.join(',') : []}.join("\n"))

__END__
| total padding (bytes) | number of structs |
|-----------------------|-------------------|
| 0                     | 16560 |
| 1                     | 81 |
| 2                     | 4 |
| 3                     | 174 |
| 4                     | 74 |
| 5                     | 19 |
| 6                     | 25 |
| 7                     | 3037 |
| 8                     | 4 |
| 11                    | 8 |
| 13                    | 2 |
| 14                    | 29 |
| 20                    | 2 |
