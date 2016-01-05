#include "benchmark.h"

using namespace annis;

std::string StaticBenchmarkDBHolder::corpus = "";
std::unique_ptr<DB> StaticBenchmarkDBHolder::db = std::unique_ptr<DB>(nullptr);
bool StaticBenchmarkDBHolder::forceFallback = true;
