package main

func buildGraph(resources []resource, processes []process, g goal) (bool, string) {
	var curr []*process

	// Maximum number of predecessors for a process. Adjust as needed.
	n := 64

	// Identify resources that are produced by every process,
	// such as "you" in the fertilizer example.
	var ubik string
	for _, resource := range resources {
		isUbik := true
		for _, process := range processes {
			found := false
			for _, product := range process.products {
				if product.name == resource.name {
					found = true
				}
			}
			if !found {
				isUbik = false
				break
			}
		}
		if isUbik {
			ubik = resource.name
			break
		}
	}

	// Find the processes that produce the goal product.
	// Can't use _, range here because we want to modify elements of the processes
	// slice while iterating over it. Range would create a copy of the slice.
	for i := range processes {
		processes[i].predecessors = make([]*process, 0, n)
		processes[i].start = -1
		processes[i].minCount = rational{numerator: 1, denominator: 1}
		for _, product := range processes[i].products {
			if product.name == g.product {
				processes[i].added = 1
				processes[i].final = true
				curr = append(curr, &processes[i])
			}
		}

		// For every ingredient, if there exists a resource with the same name
		// and non-zero quantity at least as much, then the process is initial.
		if len(processes[i].ingredients) > 0 {
			processes[i].initial = true
		}
	ingredientsLoop:
		for _, ingredient := range processes[i].ingredients {
			if ingredient.name == ubik {
				continue
			}
			found := false
			for _, resource := range resources {
				if ingredient.name == resource.name && resource.quantity > 0 && resource.quantity >= ingredient.quantity {
					found = true
					break
				}
			}
			if !found {
				processes[i].initial = false
				break ingredientsLoop
			}
		}
	}

	// Work back through the processes to build precedence relationships.
	for len(curr) > 0 {
		var next []*process
		for k := range curr {
			for _, ingredient := range curr[k].ingredients {
				for i := range processes {
					for _, product := range processes[i].products {
						if ingredient.name == product.name {
							if processes[i].name == curr[k].name || ingredient.name == ubik {
								continue
							}
							if processes[i].added > 1 {
								return false, ubik
							}
							processes[i].added++
							processes[i].successor = curr[k]
							r := rational{ingredient.quantity, 1}.Times(curr[k].minCount)
							processes[i].minCount = r.Times(rational{1, product.quantity})
							if len(curr[k].predecessors) == 0 || curr[k].predecessors[0].name != processes[i].name {
								curr[k].predecessors = append(curr[k].predecessors, &processes[i])
							}
							next = append(next, &processes[i])
							break
						}
					}
				}
			}
		}
		curr = next
	}
	return true, ubik
}
