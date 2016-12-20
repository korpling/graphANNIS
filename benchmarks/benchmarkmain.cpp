#include <celero/Celero.h>
#include <humblelogging/api.h>

#include "dynamicbenchmark.h"

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
        
        DynamicBenchmark benchmark(subdir, corpusPath, corpusName
          , true);

        QueryConfig config;

        config.numOfParallelTasks = 1;
        benchmark.registerFixture("Jobs_1", config);

        config.numOfParallelTasks = 2;
        benchmark.registerFixture("Jobs_2", config);

        config.numOfParallelTasks = 3;
        benchmark.registerFixture("Jobs_3", config);

        config.numOfParallelTasks = 4;
        benchmark.registerFixture("Jobs_4", config);

        config.numOfParallelTasks = 5;
        benchmark.registerFixture("Jobs_5", config);

        config.numOfParallelTasks = 6;
        benchmark.registerFixture("Jobs_6", config);

        config.numOfParallelTasks = 7;
        benchmark.registerFixture("Jobs_7", config);

        config.numOfParallelTasks = 8;
        benchmark.registerFixture("Jobs_8", config);
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
