#pragma once

#include <annis/db.h>

#include <string>
#include <memory>

#include <boost/thread/shared_mutex.hpp>
#include <boost/thread/lockable_adapter.hpp>

namespace annis

{

  class DBLoader : public boost::shared_lockable_adapter<boost::shared_mutex>
  {
  public:

    enum LoadStatus {
      NOT_LOADED,
      NODES_LOADED,
      FULLY_LOADED,
      numOfLoadStatus
    };

  public:
    DBLoader(std::string location);

    LoadStatus status()
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
        }
      }
      else
      {
        dbLoaded = db.load(location, true);
      }
      return db;
    }

    std::string statusString()
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

  };

}
