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

#include "threadindexjoin.h"


#include <annis/util/comparefunctions.h>
#include <annis/operators/operator.h>


using namespace annis;

ThreadIndexJoin::ThreadIndexJoin(std::shared_ptr<Iterator> lhs, size_t lhsIdx,
                     std::shared_ptr<Operator> op,
                     std::function<std::list<Annotation>(nodeid_t)> matchGeneratorFunc,
                     size_t numOfTasks, std::shared_ptr<ThreadPool> threadPool)
  : lhs(lhs), op(op), runBackgroundThreads(false), activeBackgroundTasks(0), numOfTasks(numOfTasks),
    threadPool(threadPool)
{

  results = std::unique_ptr<SharedQueue<std::vector<Match>>>(new SharedQueue<std::vector<Match>>());

  const bool operatorIsReflexive = op->isReflexive();
  lhsFetchLoop = [this, lhsIdx, matchGeneratorFunc, operatorIsReflexive]() -> void {

    std::vector<Match> currentLHSVector;

    while(this->runBackgroundThreads && this->nextLHS(currentLHSVector))
    {
      const Match& currentLHS = currentLHSVector[lhsIdx];

      std::unique_ptr<AnnoIt> itRHS = this->op->retrieveMatches(currentLHS);

      if(itRHS)
      {
        Match rhsCandidateNode;
        while(itRHS->next(rhsCandidateNode))
        {
          std::list<Annotation> rhsAnnos = matchGeneratorFunc(rhsCandidateNode.node);
          for(Annotation currentRHSAnno : rhsAnnos)
          {
            // additionally check for reflexivity
            if((operatorIsReflexive|| currentLHS.node != rhsCandidateNode.node
                   || !checkAnnotationEqual(currentLHS.anno, currentRHSAnno)))
            {
              std::vector<Match> tuple;
              tuple.reserve(currentLHSVector.size()+1);
              tuple.insert(tuple.end(), currentLHSVector.begin(), currentLHSVector.end());
              tuple.push_back({rhsCandidateNode.node, currentRHSAnno});

              this->results->push(std::move(tuple));
            }
          }
        }
      }
    }

    {
      std::lock_guard<std::mutex> lock(mutex_activeBackgroundTasks);
      activeBackgroundTasks--;

      if(activeBackgroundTasks == 0)
      {
        // if this was the last background task shutdown the queue to message that there are no more results to fetch
        this->results->shutdown();
      }
    }
  };
}

bool ThreadIndexJoin::next(std::vector<Match>& tuple)
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
        taskList.emplace_back(threadPool->enqueue(lhsFetchLoop));
      }
    }
  }



  //  wait for next item in queue or return immediatly if queue was shutdown
  return results->pop(tuple);
}

void ThreadIndexJoin::reset()
{
  runBackgroundThreads = false;

  for(auto& t : taskList)
  {
    t.wait();
  }

  lhs->reset();

  results = std::unique_ptr<SharedQueue<std::vector<Match>>>(new SharedQueue<std::vector<Match>>());
}

ThreadIndexJoin::~ThreadIndexJoin()
{
  runBackgroundThreads = false;

  for(auto& t : taskList)
  {
    t.wait();
  }
}

