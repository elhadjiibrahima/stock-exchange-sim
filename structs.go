package main

type goal struct {
	product string
	time    bool
}

type resource struct {
	name     string
	quantity int
	ubik     bool
}

type process struct {
	name         string
	ingredients  []resource
	products     []resource
	time         int
	successor    *process
	predecessors []*process
	initial      bool
	final        bool
	minCount     rational
	count        int
	start        int
	iterations   int
	doable       bool
	added        int
}

type rational struct {
	numerator   int
	denominator int
}

func (r rational) Plus(other rational) rational {
	return rational{
		numerator:   r.numerator*other.denominator + other.numerator*r.denominator,
		denominator: r.denominator * other.denominator,
	}
}

func (r rational) Times(other rational) rational {
	return rational{
		numerator:   r.numerator * other.numerator,
		denominator: r.denominator * other.denominator,
	}.simplify()
}

func (r rational) simplify() rational {
	gcd := gcd(r.numerator, r.denominator)
	if gcd == 0 {
		return rational{0, 0}
	}
	return rational{
		numerator:   r.numerator / gcd,
		denominator: r.denominator / gcd,
	}
}

// Euclid's algorithm
func gcd(a, b int) int {
	if b == 0 {
		return a
	}
	return gcd(b, a%b)
}

func lcm(a []int) int {
	if len(a) == 0 {
		return 0
	}
	m := a[0]
	for _, n := range a[1:] {
		if n == 0 || m == 0 {
			return 0
		}
		m = m * n / gcd(m, n)
	}
	return m
}
