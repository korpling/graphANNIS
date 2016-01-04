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
#include <boost/filesystem.hpp>

namespace annis {
  class DynamicBenchmark {
  public:

    DynamicBenchmark(std::string dataDir, std::string queriesDir, std::string corpusName);

    DynamicBenchmark(const DynamicBenchmark& orig) = delete;
    virtual ~DynamicBenchmark() {}
  private:
    std::string dataDir;
    std::string queriesDir;
    std::string corpus;

    void addBenchmark(const boost::filesystem::path& path);
  };
} // end namespace annis
#endif /* DYNAMICBENCHMARK_H */

