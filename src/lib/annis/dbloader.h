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

#include <annis/db.h>                         // for DB
#include <stddef.h>                           // for size_t
#include <boost/thread/lockable_adapter.hpp>  // for shared_lockable_adapter
#include <boost/thread/shared_mutex.hpp>      // for shared_mutex
#include <functional>                         // for function
#include <string>                             // for string
#include <annis/graphstorageholder.h>         // for GraphStorageHolder

namespace annis
{

  class DBLoader : public boost::upgrade_lockable_adapter<boost::shared_mutex>
  {
  public:

    enum LoadStatus {
      NOT_LOADED,
      NODES_LOADED,
      FULLY_LOADED,
      numOfLoadStatus
    };

  public:
    DBLoader(std::string location, std::function<void()> onloadCalback);

    LoadStatus status() const
    {
      if(dbLoaded)
      {
        if(db.edges.allComponentsLoaded())
        {
          return FULLY_LOADED;
        }
        else
        {
          return NODES_LOADED;
        }

      }
      return NOT_LOADED;
    }

    DB& get()
    {
      if(!dbLoaded)
      {
        dbLoaded = db.load(location, false);
        onloadCalback();
      }

      return db;
    }

    DB& getFullyLoaded()
    {
      if(dbLoaded)
      {
        if(!db.edges.allComponentsLoaded())
        {
          db.ensureAllComponentsLoaded();
          onloadCalback();
        }
      }
      else
      {
        dbLoaded = db.load(location, true);
        onloadCalback();
      }
      return db;
    }

    void unload()
    {
      dbLoaded = false;
      // clear the current data in the database
      db.clear();
    }

    size_t estimateMemorySize() const
    {
      if(dbLoaded)
      {
        return db.estimateMemorySize();
      }
      else
      {
        return 0;
      }
    }

    std::string statusString() const
    {
      switch(status())
      {
        case NOT_LOADED:
          return "NOT_LOADED";
        case NODES_LOADED:
          return "NODES_LOADED";
        case FULLY_LOADED:
          return "FULLY_LOADED";
        default:
          return "unknown";
      }
    }

  private:

    const std::string location;
    bool dbLoaded;
    DB db;

    std::function<void()> onloadCalback;

  };

}
