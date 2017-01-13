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

#include "threadnestedloop.h"

#include <annis/operators/operator.h>     // for Operator
#include <annis/util/comparefunctions.h>  // for checkAnnotationKeyEqual
#include <algorithm>                      // for move
#include "annis/iterators.h"              // for Iterator
#include "annis/types.h"                  // for Match
#include "annis/util/sharedqueue.h"       // for SharedQueue
#include "annis/util/threadpool.h"        // for ThreadPool



using namespace annis;

ThreadNestedLoop::ThreadNestedLoop(std::shared_ptr<Operator> op,
                                   std::shared_ptr<Iterator> lhs,
                                   std::shared_ptr<Iterator> rhs,
                                   size_t lhsIdx, size_t rhsIdx, bool leftIsOuter,
                                   size_t numOfTasks, std::shared_ptr<ThreadPool> threadPool)
  : op(op), outer(leftIsOuter ? lhs : rhs), firstOuterFinished(false),
    inner(leftIsOuter ? rhs : lhs), leftIsOuter(leftIsOuter),
    runBackgroundThreads(false), activeBackgroundTasks(0), numOfTasks(numOfTasks),
    threadPool(threadPool),
    initialized(false)
{

  results = std::unique_ptr<SharedQueue<std::vector<Match>>>(new SharedQueue<std::vector<Match>>());

  const bool operatorIsReflexive = op->isReflexive();

  size_t outerIdx = leftIsOuter ? lhsIdx : rhsIdx;
  size_t innerIdx = leftIsOuter ? rhsIdx : lhsIdx;

  fetchLoop = [this, outerIdx, innerIdx, leftIsOuter, operatorIsReflexive]() -> void
  {
    std::vector<Match> matchOuter;
    std::vector<Match> matchInner;

    while(this->runBackgroundThreads && this->nextTuple(matchOuter, matchInner))
    {
      bool include = true;
      // do not include the same match if not reflexive
      if(!operatorIsReflexive
         && matchOuter[outerIdx].node == matchInner[innerIdx].node
         && checkAnnotationKeyEqual(matchOuter[outerIdx].anno, matchInner[innerIdx].anno)) {
        include = false;
      }

      if(include)
      {
        if(leftIsOuter)
        {
          if(this->op->filter(matchOuter[outerIdx], matchInner[innerIdx]))
          {
            std::vector<Match> resultMatch;

            resultMatch.reserve(matchInner.size() + matchOuter.size());
            // return a tuple where the first values are from the outer relation and the iner relations tuples are added behind
            resultMatch.insert(resultMatch.end(), matchOuter.begin(), matchOuter.end());
            resultMatch.insert(resultMatch.end(), matchInner.begin(), matchInner.end());

            this->results->push(std::move(resultMatch));
          }
        }
        else
        {
          if(this->op->filter(matchInner[innerIdx], matchOuter[outerIdx]))
          {
            std::vector<Match> resultMatch;

            resultMatch.reserve(matchInner.size() + matchOuter.size());
            // return a tuple where the first values are from the inner relation and the outer relations tuples are added behind
            resultMatch.insert(resultMatch.end(), matchInner.begin(), matchInner.end());
            resultMatch.insert(resultMatch.end(), matchOuter.begin(), matchOuter.end());

            this->results->push(std::move(resultMatch));
          }
        }
      }
    } // end while outer

    {
      std::lock_guard<std::mutex> lock(mutex_activeBackgroundTasks);
      activeBackgroundTasks--;

      if(activeBackgroundTasks == 0)
      {
        // if this was the last background task shutdown the queue to message that there are no more results to fetch
        this->results->shutdown();
      }
    }


  }; // end fetchLoop function
}

bool ThreadNestedLoop::next(std::vector<Match>& tuple)
{
  if(!runBackgroundThreads)
  {
    {
      runBackgroundThreads = true;
      // Make sure activeBackgroundTasks is correct before actually running all the threads.
      // Thus if a thread immediatly returns since there is no result only the very last
      // thread will trigger a shutdown.
      {
        std::lock_guard<std::mutex> lock(mutex_activeBackgroundTasks);
        activeBackgroundTasks = numOfTasks;
      }

      if(!threadPool)
      {
        threadPool = std::make_shared<ThreadPool>(numOfTasks);
      }

      for(size_t i=0; i < numOfTasks; i++)
      {
        taskList.emplace_back(threadPool->enqueue(fetchLoop));
      }
    }
  }



  //  wait for next item in queue or return immediatly if queue was shutdown
  return results->pop(tuple);
}

bool ThreadNestedLoop::nextTuple(std::vector<Match> &matchOuter, std::vector<Match> &matchInner)
{
  std::lock_guard<std::mutex> lock(mutex_fetch);

  bool proceed = true;

  if(!initialized)
  {
    proceed = false;
    if(outer->next(currentOuter))
    {
      proceed = true;
      initialized = true;
    }
  }

  while(proceed)
  {
    while(fetchNextInner(matchInner))
    {
      matchOuter = currentOuter;
      return true;
    } // end for each inner

    if(outer->next(currentOuter))
    {
      firstOuterFinished = true;
      itInnerCache = innerCache.begin();
      inner->reset();

      if(innerCache.empty())
      {
        // inner is always empty, no point in trying to get more from the outer side
        proceed = false;
      }

    }
    else
    {
      proceed = false;
    }
  }

  return false;

}

void ThreadNestedLoop::reset()
{
  runBackgroundThreads = false;


  for(auto& t : taskList)
  {
    t.wait();
  }

  inner->reset();
  outer->reset();
  innerCache.clear();
  itInnerCache = innerCache.begin();
  firstOuterFinished = false;
  initialized = false;

  results = std::unique_ptr<SharedQueue<std::vector<Match>>>(new SharedQueue<std::vector<Match>>());
}

ThreadNestedLoop::~ThreadNestedLoop()
{
  runBackgroundThreads = false;

  for(auto& t : taskList)
  {
    t.wait();
  }
}



