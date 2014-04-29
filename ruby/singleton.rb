# module Container
	# def fields() @fields ||= [] end
# end

class Stream
	class << self
		def fields() @fields ||= [] end
	end

	def initialize(*ary)
		this = singleton_class
		this = class << self; self end
		singleton_class.fields.push *ary
		#this.instance_eval { fields.push *ary }
	end

	def initialize_copy(other)
		singleton_class.fields.replace(other.singleton_class.fields)
		super;
	end

	def fields
		# which one
		c = class << self; self end
		[self.class.fields, self.singleton_class.fields, c.fields]
	end
end

o = Stream.new(:a, :b)
p o.fields
p Stream.new(2, 3).fields
p o.fields
p o.clone.fields
p o.clone.singleton_class.fields
# Ruby 1.9.3: => []
# Ruby 2.0.0: => [:a, :b]