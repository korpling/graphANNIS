#include "benchmark.h"

using namespace annis;

std::string BenchmarkDBHolder::corpus = "";
std::unique_ptr<DB> BenchmarkDBHolder::db = std::unique_ptr<DB>(nullptr);
bool BenchmarkDBHolder::forceFallback = true;
