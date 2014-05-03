require 'pp'
n=95
v='07700 900 334';v.succ!until v.tr('^34',?0).sum==634&&(n-=1)<1;p v
all = (0..9).to_a.repeated_combination(4).map {|n| "07700 90%d %d%d%d" % n }
v='07700 900 334';v.succ!until v.tr('^34',?0).sum==634&&(n-=1)<1;v
#v='07700 90%d %d';(v%334..v%999).select{|v|v.tr('^34',?!).sum==220}[n-1]
#('07700 900 334'..?1*13).select{|v|v.tr('^34',?!).sum==220}[n-1]
#[*(0..9)].combination(4).map{|a|"07700 90%d %d%d%d"}%.flat_map{|i|[*[*[i,3,4].uniq,3].permutation(4)]}.uniq.sort[n-1]

pp [*0..2,*5..9].flat_map{|i|[*[i,3,3,4].permutation]}.uniq.sort.map.with_index {|a, i|
	d = a.join.to_i-334
	n=i+1
	[n, n/3, *d.to_s.rjust(4,?0).split('').map(&:to_i)]
}.each_slice(3).to_a
#p (0..9).to_a.repeated_combination(4).select {|a| a.join.tr('^34',?0).sum==202 }.map.with_index {|a, i| [i+1, a.join.to_i-334] }
