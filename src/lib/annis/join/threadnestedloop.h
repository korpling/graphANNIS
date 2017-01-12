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
#include <annis/types.h>      // for Match
#include <stddef.h>           // for size_t
#include <atomic>             // for atomic_bool
#include <deque>              // for deque, _Deque_iterator, deque<>::const_...
#include <functional>         // for function
#include <future>             // for future
#include <memory>             // for shared_ptr, __shared_ptr, unique_ptr
#include <mutex>              // for mutex
#include <vector>             // for vector, allocator
namespace annis { class Operator; }  // lines 36-36
namespace annis { class ThreadPool; }
namespace annis { template <typename T> class SharedQueue; }


namespace annis
{

class ThreadNestedLoop : public Iterator
{
public:
  struct MatchPair
  {
    Match lhs;
    Match rhs;
  };


public:
  ThreadNestedLoop(
            std::shared_ptr<Operator> op,
            std::shared_ptr<Iterator> lhs, std::shared_ptr<Iterator> rhs,
            size_t lhsIdx, size_t rhsIdx,
            bool leftIsOuter,
            size_t numOfTasks,
            std::shared_ptr<ThreadPool> threadPool = std::shared_ptr<ThreadPool>());

  virtual bool next(std::vector<Match>& tuple) override;
  virtual void reset() override;

  virtual ~ThreadNestedLoop();
private:

  std::shared_ptr<Operator> op;

  std::shared_ptr<Iterator> outer;
  bool firstOuterFinished;

  std::shared_ptr<Iterator> inner;
  std::deque<std::vector<Match>> innerCache;
  std::deque<std::vector<Match>>::const_iterator itInnerCache;


  const bool leftIsOuter;

  std::atomic_bool runBackgroundThreads;
  size_t activeBackgroundTasks;
  std::mutex mutex_activeBackgroundTasks;

  const size_t numOfTasks;
  std::shared_ptr<ThreadPool> threadPool;

  std::unique_ptr<SharedQueue<std::vector<Match>>> results;
  std::function<void()> fetchLoop;

  std::deque<std::future<void>> taskList;

  std::mutex mutex_fetch;
  bool initialized;
  std::vector<Match> currentOuter;

private:

  bool nextTuple(std::vector<Match>& matchOuter, std::vector<Match>& matchInner);

  bool fetchNextInner(std::vector<Match>& matchInner)
  {
    if(firstOuterFinished)
    {
      if(itInnerCache != innerCache.end())
      {
        matchInner = *itInnerCache;
        itInnerCache++;
        return true;
      }
      else
      {
        return false;
      }
    }
    else
    {
      bool hasNext = inner->next(matchInner);
      if(hasNext)
      {
        innerCache.push_back(matchInner);
      }
      return hasNext;
    }
  }



};
}

