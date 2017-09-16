/*
   Copyright 2017 Thomas Krause <thomaskrause@posteo.de>

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

#include <celero/Celero.h>
#include <humblelogging/api.h>

#include "dynamicbenchmark.h"

#include <annis/util/threadpool.h>

using namespace annis;

int main(int argc, char **argv) {
  
  humble::logging::Factory &fac = humble::logging::Factory::getInstance();
  fac.setConfiguration(humble::logging::DefaultConfiguration::createFromString(
          "logger.level(*)=info\n"
          ));
  fac.setDefaultFormatter(new humble::logging::PatternFormatter("[%date]- %m (%lls, %filename:%line)\n"));
  fac.registerAppender(new humble::logging::FileAppender("benchmark_annis4.log", true));
  
  if(argc > 1)
  {
    std::string benchmarkDir(argv[1]);
    
    // find all sub-directories of the "queries" folder
    boost::filesystem::directory_iterator fileEndIt;
    boost::filesystem::directory_iterator itFiles(benchmarkDir + "/queries");
    while(itFiles != fileEndIt)
    {
      if(itFiles->status().type() == boost::filesystem::directory_file)
      {
        const auto subdirPath = itFiles->path();
        std::string subdir = subdirPath.string();
        std::string corpusName = subdirPath.filename().string();
        
        // get the corpus path (is subfolder of "data" folder)
        std::string corpusPath = benchmarkDir + "/data/" + corpusName;
        
        DynamicBenchmark benchmark(subdir, corpusPath, corpusName, 6000000, true);

        {
          // default configuration and all optimizations enabled but no parallization
          QueryConfig config;
          benchmark.registerFixture("default", config);
        }

        {
          // No optimized graph storages
          QueryConfig config;
          config.forceFallback = true;
          benchmark.registerFixture("force_fallback", config);
        }

        {
          // no query optimization at all
          QueryConfig config;
          config.optimize = false;
          benchmark.registerFixture("no_optimization", config);
        }

        {
          // no operand order optimization
          QueryConfig config;
          config.optimize_operand_order = false;
          benchmark.registerFixture("no_operand_order", config);
        }

        {
          // no unbound regex optimization
          QueryConfig config;
          config.optimize_operand_order = false;
          benchmark.registerFixture("no_unbound_regex", config);
        }

        {
          // no node by edge annotation search
          QueryConfig config;
          config.optimize_nodeby_edgeanno = false;
          benchmark.registerFixture("no_nodeby_edgeanno", config);
        }

        {
          // no join order optimization
          QueryConfig config;
          config.optimize_join_order = false;
          benchmark.registerFixture("no_join_order", config);
        }

        {
          // not using all permutations in join order optimization
          QueryConfig config;
          config.all_permutations_threshold = 0;
          benchmark.registerFixture("no_join_order_permutation", config);
        }


        // add different parallel configurations for threads and SIMD (+thread)
        unsigned int numOfCPUs = std::thread::hardware_concurrency();
        std::shared_ptr<ThreadPool> sharedThreadPool = std::make_shared<ThreadPool>(numOfCPUs);

        for(int i=2; i <= numOfCPUs; i += 2)
        {
          QueryConfig config;
          config.threadPool = i > 0 ? sharedThreadPool : nullptr;
          config.numOfBackgroundTasks = i;
          config.enableSIMDIndexJoin = false;
          config.enableThreadIndexJoin = true;
          benchmark.registerFixture("threads_" + std::to_string(i), config);
        }

        for(int i=0; i <= numOfCPUs; i += 2)
        {
          QueryConfig config;
          config.threadPool = i > 0 ? sharedThreadPool : nullptr;
          config.numOfBackgroundTasks = i;
          config.enableSIMDIndexJoin = true;
          config.enableThreadIndexJoin = i == 0 ? false : true;
          benchmark.registerFixture("simd_" + std::to_string(i), config);
        }

      }
      itFiles++;
    }
    
    
    celero::Run(argc, argv);
    
  }
  else
  {
    std::cout << "You have to give a benchmark directy (which contains a \"queries\" and \"data\" sub-directory) as argument."
      << std::endl;
  }

  return 0;
}
