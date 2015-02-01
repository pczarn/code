require 'pp'

s = File.read(ARGV.first || 'layout.txt')

crates = {}
s.split(/^(?=rustc)/).drop(1).each do |crate|
   lib = crate[/\/(\w+)$/, 1]
   structs = crate.scan(/(\d+) size (\d+)-(\d+): \[(.*)\]/)
         .map do |id, sz, pad, fsz|
      # struct DefId { krate: 2, node: 133725 } size 96-3: [8, 4, 1, 16, 16, 16, 16, 16]
      [id, sz.to_i, pad.to_i, fsz.split(', ').map(&:to_i)]
   end
   crates[lib] = structs
end

# sz_csv = {}
# al_csv = {}
pad_stats = {}
# sz_csv.default = al_csv.default =
pad_stats.default = 0

p crates.size, crates.map{|lib,_|lib}

crates.values.flatten(1).uniq.map do |*head, fsizes|
   [*head, fsizes.inject(&:+) || 0]
end.compact
   .each do |_, sz, pad, fsz|
   pad_stats[pad] += 1
end

puts pad_stats.sort_by(&:first).flat_map{|a|a ? a.join(' | ') : []}.join("\n")

# File.write('layout_sz.csv', sz_csv.flat_map{|a|a ? a.join(',') : []}.join("\n"))
# File.write('layout_al.csv', al_csv.flat_map{|a|a ? a.join(',') : []}.join("\n"))

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
-
0 | 1621
1 | 10
2 | 5
3 | 100
4 | 83
5 | 9
6 | 38
7 | 209
8 | 8
10 | 2
11 | 53
12 | 4
13 | 3
14 | 44
15 | 4
17 | 2
18 | 9
19 | 1
21 | 7
22 | 5
24 | 2
25 | 3
26 | 1
27 | 1
28 | 2
29 | 1
31 | 1
32 | 2
33 | 8
35 | 10
40 | 1
42 | 2
44 | 3
47 | 1
53 | 1
55 | 5
92 | 1
106 | 1
110 | 1
631 | 1
675 | 2
963 | 1