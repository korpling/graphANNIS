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
#include <atomic>             // for atomic_bool
#include <deque>              // for deque
#include <functional>         // for function
#include <future>             // for future
#include <list>               // for list
#include <memory>             // for shared_ptr, __shared_ptr, unique_ptr
#include <mutex>              // for mutex, lock_guard
#include <vector>             // for vector
namespace annis { class Operator; }  // lines 36-36
namespace annis { class ThreadPool; }
namespace annis { template <typename T> class SharedQueue; }

namespace annis
{

class ThreadIndexJoin : public Iterator
{
public:
  struct MatchPair
  {
    Match lhs;
    Match rhs;
  };


public:
  ThreadIndexJoin(std::shared_ptr<Iterator> lhs, size_t lhsIdx,
            std::shared_ptr<Operator> op,
            std::function<std::list<Annotation>(nodeid_t)> matchGeneratorFunc,
            size_t numOfTasks = 1,
            std::shared_ptr<ThreadPool> threadPool = std::shared_ptr<ThreadPool>());

  virtual bool next(std::vector<Match>& tuple) override;
  virtual void reset() override;

  virtual ~ThreadIndexJoin();
private:

  std::shared_ptr<Iterator> lhs;
  std::mutex mutex_lhs;

  std::shared_ptr<Operator> op;



  std::atomic_bool runBackgroundThreads;
  size_t activeBackgroundTasks;
  std::mutex mutex_activeBackgroundTasks;
  const size_t numOfTasks;
  std::shared_ptr<ThreadPool> threadPool;

  std::unique_ptr<SharedQueue<std::vector<Match>>> results;
  std::function<void()> lhsFetchLoop;

  std::deque<std::future<void>> taskList;

private:
  bool nextLHS(std::vector<Match>& tuple)
  {
    std::lock_guard<std::mutex> lock(mutex_lhs);
    return lhs->next(tuple);
  }

};
}

