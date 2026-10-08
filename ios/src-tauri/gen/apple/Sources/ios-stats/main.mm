#include "bindings/bindings.h"

int main(int argc, char * argv[]) {
	ffi::ios_stats_register_background_refresh();
	ffi::start_app();
	return 0;
}
