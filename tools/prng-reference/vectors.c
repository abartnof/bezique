/* Prints reference outputs for SplitMix64 and xoshiro256**, using Vigna's
 * reference implementations unchanged (splitmix64.c, xoshiro256starstar.c,
 * from https://prng.di.unimi.it/). The Rust generator in
 * crates/bezique-core/src/rng.rs pins these numbers in its tests.
 *
 *   cc -O2 -o vectors vectors.c && ./vectors
 *
 * Each reference file defines a global `next`, so each is included in its
 * own namespace by renaming its symbols with the preprocessor. */
#include <stdint.h>
#include <stdio.h>

#define next splitmix_next
#define x splitmix_state
#include "splitmix64.c"
#undef next
#undef x

#define next xoshiro_next
#define jump xoshiro_jump
#define long_jump xoshiro_long_jump
#include "xoshiro256starstar.c"
#undef next

/* Seeds xoshiro256** the way its author recommends: four outputs of
 * SplitMix64 started from the 64-bit seed. */
static void seed_xoshiro(uint64_t seed) {
	splitmix_state = seed;
	for (int i = 0; i < 4; i++) s[i] = splitmix_next();
}

int main(void) {
	const uint64_t seeds[] = { 0, 42, 20261001 };
	for (int k = 0; k < 3; k++) {
		splitmix_state = seeds[k];
		printf("splitmix64 seed %llu:", (unsigned long long)seeds[k]);
		for (int i = 0; i < 4; i++) printf(" 0x%016llx", (unsigned long long)splitmix_next());
		printf("\n");
	}
	for (int k = 0; k < 3; k++) {
		seed_xoshiro(seeds[k]);
		printf("xoshiro256** seed %llu:", (unsigned long long)seeds[k]);
		for (int i = 0; i < 6; i++) printf(" 0x%016llx", (unsigned long long)xoshiro_next());
		printf("\n");
	}
	return 0;
}
