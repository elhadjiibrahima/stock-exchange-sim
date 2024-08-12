package main

import (
	"fmt"
	"strings"
)

func schedule(resources map[string]int,
	processes []process,
	finite bool,
	c <-chan struct{},
	ubik string) (int, string) {
	finishedAt := 0
	curr := make([]*process, 0, len(processes))
	var builder strings.Builder
	pass := 0

	for i := range processes {
		processes[i].count = processes[i].minCount.numerator
		if processes[i].initial {
			processes[i].start = 0
			processes[i].count = processes[i].minCount.numerator
			processes[i].doable = true
		}
	}

	arr := make([]int, 0, len(processes))
	for j := range processes {
		arr = append(arr, processes[j].minCount.denominator)
	}
	maxCount := lcm(arr)
	for i := range processes {
		processes[i].count = processes[i].minCount.Times(rational{maxCount, 1}).numerator
	}

	// Infinite loop limited by timer. See checkArgs() in stock.go
	// for default timer value.
pass:
	for ; ; pass++ {
		for i := range processes {
			if processes[i].initial && processes[i].doable {
				isSufficient := true
				for _, ingredient := range processes[i].ingredients {
					less := resources[ingredient.name] < ingredient.quantity*processes[i].count
					if less || resources[ingredient.name] == 0 {
						isSufficient = false
					}
				}
				if !isSufficient {
					processes[i].doable = false
					continue
				}
				curr = append(curr, &processes[i])
			}
		}

		if len(curr) == 0 {
			break
		}

		for len(curr) > 0 {
			select {
			case <-c:
				return finishedAt, builder.String()
			default:
			}
			next := make([]*process, 0, len(processes))
			added := make(map[string]bool)

			for i := range curr {
				isSufficient := true
				for _, ingredient := range curr[i].ingredients {
					less := resources[ingredient.name] < ingredient.quantity*curr[i].count
					if less || resources[ingredient.name] == 0 {
						isSufficient = false
					}
				}
				if !isSufficient {
					continue
				}
				curr[i].iterations += curr[i].count
				for _, ingredient := range curr[i].ingredients {
					if ingredient.name != ubik {
						resources[ingredient.name] -= ingredient.quantity * curr[i].count
					}
				}
				for _, product := range curr[i].products {
					if product.name != ubik {
						resources[product.name] += product.quantity * curr[i].count
					}
				}
				if curr[i].successor != nil {
					end := curr[i].start + curr[i].time
					if end > curr[i].successor.start {
						curr[i].successor.start = end
					}
					builder.WriteString(fmt.Sprintf(" %d:%s\n", curr[i].start, curr[i].name))
					if !finite && curr[i].successor.initial {
						curr = []*process{}
						continue pass
					}
					if !added[curr[i].successor.name] {
						added[curr[i].successor.name] = true
						next = append(next, curr[i].successor)
					}
				}

				if finite && curr[i].final {
					finishedAt = curr[i].start + curr[i].time
				}
				if !finite {
					finishedAt = curr[i].start + maxTime(curr)
				}
			}
			curr = next
		}
	}
	return finishedAt, builder.String()
}

func maxTime(processes []*process) int {
	max := 0
	for i := range processes {
		if processes[i].time > max {
			max = processes[i].time
		}
	}
	return max
}
