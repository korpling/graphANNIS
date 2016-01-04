/*
 * To change this license header, choose License Headers in Project Properties.
 * To change this template file, choose Tools | Templates
 * and open the template in the editor.
 */

/* 
 * File:   DynamicBenchmark.h
 * Author: thomas
 *
 * Created on 4. Januar 2016, 11:54
 */

#ifndef DYNAMICBENCHMARK_H
#define DYNAMICBENCHMARK_H

#include "benchmark.h"

#include <humblelogging/api.h>
#include <celero/Celero.h>
#include <boost/filesystem.hpp>
#include <boost/format.hpp>

namespace annis {
  class DynamicBenchmark {
  public:

    DynamicBenchmark(std::string dataDir, std::string queriesDir, std::string corpusName)
    : dataDir(dataDir), queriesDir(queriesDir), corpus(corpusName) {

      // find all file ending with ".json" in the folder
      boost::filesystem::directory_iterator fileEndIt;

      boost::filesystem::directory_iterator itFiles(queriesDir);
      while (itFiles != fileEndIt) {
        const auto filePath = itFiles->path();
        if (filePath.extension().string() == ".json") {
          addBenchmark(filePath);
        }
        itFiles++;
      }
    }

    DynamicBenchmark(const DynamicBenchmark& orig) = delete;
    virtual ~DynamicBenchmark() {}
  private:
    humble::logging::Logger& logger = humble::logging::Factory::getInstance().getLogger("default");
    std::string dataDir;
    std::string queriesDir;
    std::string corpus;

    void addBenchmark(const boost::filesystem::path& path)
    {
      HL_INFO(logger, (boost::format("adding benchmark %1%") % path.string()).str());
      // create fallback fixture
    }
  };
} // end namespace annis
#endif /* DYNAMICBENCHMARK_H */

