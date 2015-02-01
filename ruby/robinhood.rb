require 'bigdecimal'

class Integer
  # binomial coefficient: n C k
  def choose(k)
    # n!/(n-k)!
    pTop = (self-k+1 .. self).inject(1, &:*) 
    # k!
    pBottom = (2 .. k).inject(1, &:*)
    pTop / pBottom
  end
end

def fact(n) (1..n).inject(1, :*) end

MEMO = {}

def t(k, d, b)
   if d >= b or d < 0 or k < 0
      0
   elsif k == 0
      1
   else
      hsh = (k.to_i << 32) + (d.to_i << 16) + b.to_i
      if r = MEMO[hsh]
         return r
      end

      r = - (0...k).each.lazy.map{|j|
         k.choose(j) * (((k+d).to_f/b).floor**(k-j)) * t(j,d,b)
      }.inject(0, :+)

      MEMO[hsh] = r
   end
end

def tz(d, b, z)
   (0..100).each.lazy.map {|k|
      t(k, d, b) * z**k / fact(k)
   }.inject(:+)
end

def fn(b, a, k, n, d, s)
   (k*b*a)**(b*(n+k)+d-s) / fact(b*(n+k)+d-s)
end

def psi(n, b, a)
   e = Math::E
   ba = b*a;
   (0...b).map {|s|
      p = tz(b - 1 - s, b, ba) / ba

      q = (1...100).map {|k|
         (e**(-k*ba)) * (0...b).map {|d|
            fn(b, a, k, n, d, s) - fn(b, a, k, n+1, d, s)
         }.inject(0, :+)
      }

      # p p, q
      p * q.inject(0, :+)
   }.inject(0, :+)
end

a = 0.95
# ary = (0..50).map {|n| psi(BigDecimal::new(n), 1, BigDecimal::new(a, 10)) }
# puts (0..50).map {|n| "#{n} => #{ary[n]}, sum = #{ary[n..25].inject(:+)}   f #{ary[n]/ary[n+1] if ary[n+1]}" }


# puts (8..20).map {|n| psi(n, 1, a) }.inject(:+)

# puts (0..20).map{|n| psi(n, 1, 0.8) }.inject(:+)
# puts (0..20).map{|n| "#{n+1} => #{psi(n, 1, 0.9)}" }
# puts (0..20).map{|n| "#{n+1} => #{psi(n, 1, 0.95)}" }

# (0.4 .. 0.9).step()

t = [1]

# (1...500).each do |k|
#    p = 1
#    bn = 1
#    t.push -t.reverse.each_with_index.map {|t_j, j|
#        bn = bn * (k - j) / (j + 1) # k.choose(j+1)
#        p *= k
#        bn * p * t_j
#    }.inject(0, :+)
# end

T = t

def psi_opt2(n, a)
   e = Math::E
   # e = BigDecimal::new(Math::E, 10)

   p = 1 / a - 1

   # p = T.each_with_index.map {|t_k, k|
   #    t_k * a**(k-1) / fact(k)
   # }.inject(0, :+)

   q = (1...1500).map {|k|
      pwr = [(k*a).floor, k+n]
      ka_over_e = k*a/e
      ka = k*a

      r = (1..pwr.min).map {|p|
         ka_over_e/p
      } + (pwr.min+1 .. k+n).map {|p|
         ka/p
      }

      r.reverse.inject(1, :*) * e**-(k*a-pwr.min) * (1 - k*a/(n + k + 1))
   }

   # p p, q
   p * q.inject(0, :+)
end

def psi_opt3(n, a)
   e = Math::E
   # e = BigDecimal::new(Math::E, 10)

   p = 1 / a - 1
   q = (1...100).map {|k|
      pwr = [(k*a).floor, k+n]
      (k*a)**(k+n) * (e**(-(k*a))) / fact(k + n) * (
         1.to_f - (k*a)/(k + n + 1)
      )
   }
   p * q.inject(:+)
end

def psi_opt(n, a)
   e = Math::E

   p = 1 / a - 1

   # p = (0..20).each.map {|k|
   #    t(k, 0, 1) * a**(k-1) / fact(k)
   # }

   q = (1...100).map {|k|
      (e**(-k*a)) * (k*a)**(k+n) / fact(k + n) * (
         1.to_f - (k*a) / (k + n + 1)
      )
   }

   # p p, q
   p * q.inject(0, :+)
end

a = 0.909

prob = 100.times.map {|n| psi_opt2(n, a) }

# puts psi_opt2(10, BigDecimal::new(a))
puts prob, prob.inject(:+), prob[0..3].inject(:+), prob[0..7].inject(:+)
# puts psi_opt(10, 0.6)
# puts psi(10, 1, 0.95)

# puts T
# puts t(0, 0, 1), t(1, 0, 1), t(2, 0, 1), t(3, 0, 1), t(100, 0, 1), t(200, 0, 1)

# 10, 0.95 => 0.0293533090357552505431913736193743933647354805008
#             0.029353307994242694


# 0.08090420132441722
# 0.08406587259056261
# 0.0777561863394268
# 0.07029125388465278
# 0.06337353742316414
# 0.057128495902749335
# 0.05149544020047412
# 0.046413291840851356

# 0.04182829592328831
# 0.03769211929468622
# 0.03396111192920974
# 0.03059584996398247
# 0.027560729546883968
# 0.02482359768432502
# 0.02235541847449426
# 0.020129971607272253

# | X                 | Pr{X}      |
# |-------------------|------------|
# | displacement = 0  | 0.08090420 |
# | displacement = 1  | 0.08406587 |
# | displacement = 2  | 0.07775618 |
# | displacement = 3  | 0.07029125 |
# | displacement = 4  | 0.06337353 |
# | displacement = 5  | 0.05712849 |
# | displacement = 6  | 0.05149544 |
# | displacement = 7  | 0.04641329 |
# | displacement = 8  | 0.04182829 |
# | displacement = 9  | 0.03769211 |
# | displacement = 10 | 0.03396111 |
# | displacement = 11 | 0.03059584 |
# | displacement = 12 | 0.02756072 |
# | displacement = 13 | 0.02482359 |
# | displacement = 14 | 0.02235541 |
# | displacement = 15 | 0.02012997 |
# |                   |            |
# | displacement < 8  | 0.53142827 |
# | displacement < 16 | 0.77037537 |