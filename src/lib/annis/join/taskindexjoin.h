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

#pragma once

#include <annis/iterators.h>  // for Iterator
#include <annis/types.h>      // for Match, nodeid_t
#include <stddef.h>           // for size_t
#include <functional>         // for function
#include <future>             // for future
#include <list>               // for list
#include <memory>             // for shared_ptr, make_shared
#include <thread>             // for thread
#include <vector>             // for vector
namespace annis { class Operator; }  // lines 36-36
namespace annis { class ThreadPool; }


namespace annis
{

class TaskIndexJoin : public Iterator
{
public:

  struct MatchPair
  {
    std::vector<Match> lhs;
    Match rhs;
  };

public:
  TaskIndexJoin(std::shared_ptr<Iterator> lhs, size_t lhsIdx,
            std::shared_ptr<Operator> op,
            std::function<std::list<Annotation> (nodeid_t)> matchGeneratorFunc,
            unsigned maxNumfOfTasks = std::thread::hardware_concurrency(),
            std::shared_ptr<ThreadPool> threadPool = std::shared_ptr<ThreadPool>());

  virtual bool next(std::vector<Match>& tuple) override;
  virtual void reset() override;

  virtual ~TaskIndexJoin();
private:

  std::shared_ptr<Iterator> lhs;
  const size_t lhsIdx;
  const unsigned maxNumfOfTasks;

  std::shared_ptr<ThreadPool> workerPool;

  std::list<std::future<std::list<MatchPair>>> taskBuffer;
  size_t taskBufferSize;
  std::list<MatchPair> matchBuffer;

  std::function<std::list<MatchPair>(const std::vector<Match>& )> taskBufferGenerator;

private:
  bool fillTaskBuffer();
  bool nextMatchBuffer();
};
}

