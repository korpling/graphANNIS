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

#include "graphstorageholder.h"

#include <annis/graphstorage/adjacencyliststorage.h>    // for AdjacencyList...
#include <annis/util/helper.h>                          // for Helper
#include <humblelogging/api.h>                          // for HL_DEBUG, HUM...
#include <humblelogging/logger.h>                       // for Logger
#include <boost/filesystem/operations.hpp>              // for directory_ite...
#include <boost/filesystem/path.hpp>                    // for path, operator/
#include <boost/format.hpp>
#include <boost/iterator/iterator_facade.hpp>           // for iterator_facade
#include <boost/optional/optional.hpp>                  // for get_pointer
#include <boost/thread/pthread/condition_variable.hpp>  // for interruption_...
#include <cereal/archives/binary.hpp>                   // for BinaryInputAr...
#include <cereal/cereal.hpp>                            // for InputArchive
#include <sstream>                                      // for operator<<
#include <utility>                                      // for pair
#include "annis/graphstorage/graphstorage.h"            // for ReadableGraph...
#include "annis/graphstorageregistry.h"                 // for GraphStorageR...
namespace annis { class StringStorage; }


HUMBLE_LOGGER(logger, "annis4");

using namespace annis;


GraphStorageHolder::GraphStorageHolder(StringStorage &strings)
  : strings(strings)
{

}

GraphStorageHolder::~GraphStorageHolder()
{

}

void GraphStorageHolder::clear()
{
  container.clear();
}

std::shared_ptr<const ReadableGraphStorage> GraphStorageHolder::getGraphStorage(const Component &component)
{
  std::map<Component, std::shared_ptr<ReadableGraphStorage>>::const_iterator itGS = container.find(component);
  if(itGS != container.end())
  {
    ensureComponentIsLoaded(itGS->first);
    return itGS->second;
  }
  return std::shared_ptr<const ReadableGraphStorage>();
}


std::shared_ptr<const ReadableGraphStorage> GraphStorageHolder::getGraphStorage(ComponentType type, const std::string &layer, const std::string &name)
{
  Component c = {type, layer, name};
  return getGraphStorage(c);
}

std::vector<std::shared_ptr<const ReadableGraphStorage> > GraphStorageHolder::getGraphStorage(ComponentType type, const std::string &name)
{
  std::vector<std::shared_ptr<const ReadableGraphStorage> > result;

  Component componentKey;
  componentKey.type = type;
  componentKey.layer[0] = '\0';
  componentKey.name[0] = '\0';

  for(auto itGS = container.lower_bound(componentKey);
      itGS != container.end() && itGS->first.type == type;
      itGS++)
  {
    const Component& c = itGS->first;
    if(name == c.name)
    {
      ensureComponentIsLoaded(itGS->first);
      result.push_back(itGS->second);
    }
  }

  return result;
}

std::vector<std::shared_ptr<const ReadableGraphStorage> > GraphStorageHolder::getGraphStorage(ComponentType type)
{
  std::vector<std::shared_ptr<const ReadableGraphStorage>> result;

  Component c;
  c.type = type;
  c.layer[0] = '\0';
  c.name[0] = '\0';

  for(
     std::map<Component,std::shared_ptr<ReadableGraphStorage>>::const_iterator itGS = container.lower_bound(c);
      itGS != container.end() && itGS->first.type == type;
      itGS++)
  {
    ensureComponentIsLoaded(itGS->first);
    result.push_back(itGS->second);
  }

  return result;
}

size_t GraphStorageHolder::estimateMemorySize() const
{
  size_t result = 0;
  for(std::pair<Component, std::shared_ptr<ReadableGraphStorage>> e : container)
  {
    if(e.second)
    {
      result += e.second->estimateMemorySize();
    }
  }
  return result;
}

std::string GraphStorageHolder::info()
{
  std::stringstream ss;
  for(GraphStorageIt it = container.begin(); it != container.end(); it++)
  {
    const Component& c = it->first;
    const std::shared_ptr<ReadableGraphStorage> gs = it->second;

    if(!gs)
    {
      ss << "Component " << debugComponentString(c) << std::endl << "(not loaded yet)" << std::endl;
    }
    else
    {
      ss << "Component " << debugComponentString(c) << ": " << gs->numberOfEdges() << " edges and "
         << gs->numberOfEdgeAnnotations() << " annotations" << std::endl;

      std::string implName = registry.getName(gs);
      if(!implName.empty())
      {
        ss << "implementation: " << implName << std::endl;
        ss << "estimated size: " << Helper::inMB(gs->estimateMemorySize()) << " MB" << std::endl;
      }

      GraphStatistic stat = gs->getStatistics();
      if(stat.valid)
      {
        ss << "nodes: " << stat.nodes << std::endl;
        ss << "fan-out: " << stat.avgFanOut << " (avg) / " << stat.maxFanOut << " (max)" << std::endl;
        if(stat.cyclic)
        {
          ss << "cyclic" << std::endl;
        }
        else
        {
          ss << "non-cyclic, max. depth: " << stat.maxDepth << ", DFS visit ratio: " << stat.dfsVisitRatio << std::endl;

        }
        if(stat.rootedTree)
        {
          ss << "rooted tree" << std::endl;
        }
      }
    }
    ss << "--------------------" << std::endl;
  }
  return ss.str();
}

bool GraphStorageHolder::load(std::string dirPath, bool preloadComponents)
{
  clear();
  boost::filesystem::directory_iterator fileEndIt;

  for(unsigned int componentType = (unsigned int) ComponentType::COVERAGE;
      componentType < (unsigned int) ComponentType::ComponentType_MAX; componentType++)
  {
    const boost::filesystem::path componentPath(dirPath + "/gs/"
                                                + ComponentTypeHelper::toString((ComponentType) componentType));

    if(boost::filesystem::is_directory(componentPath))
    {
      // get all the namespaces/layers
      boost::filesystem::directory_iterator itLayers(componentPath);
      while(itLayers != fileEndIt)
      {
        const boost::filesystem::path layerPath = *itLayers;



        // try to load the component with the empty name
        Component emptyNameComponent = {(ComponentType) componentType,
            layerPath.filename().string(), ""};

        std::shared_ptr<ReadableGraphStorage> gsEmptyName;

        if(preloadComponents)
        {
          HL_DEBUG(logger, (boost::format("loading component %1%")
                           % debugComponentString(emptyNameComponent)).str());
          auto inputFile = layerPath / "component.cereal";
          std::ifstream is(inputFile.string(), std::ios::binary);
          if(is.is_open())
          {
            cereal::BinaryInputArchive ar(is);
            ar(gsEmptyName);
          }
        }
        else
        {
          notLoadedLocations.insert({emptyNameComponent, layerPath.string()});
        }
        container[emptyNameComponent] = gsEmptyName;


        // also load all named components
        boost::filesystem::directory_iterator itNamedComponents(layerPath);
        while(itNamedComponents != fileEndIt)
        {
          const boost::filesystem::path namedComponentPath = *itNamedComponents;
          if(boost::filesystem::is_directory(namedComponentPath))
          {
            // try to load the named component
            Component namedComponent = {(ComponentType) componentType,
                                                           layerPath.filename().string(),
                                                           namedComponentPath.filename().string()
                                       };


            std::shared_ptr<ReadableGraphStorage> gsNamed;
            if(preloadComponents)
            {
              HL_DEBUG(logger, (boost::format("loading component %1%")
                               % debugComponentString(namedComponent)).str());
              auto inputFile = namedComponentPath / "component.cereal";
              std::ifstream is(inputFile.string(), std::ios::binary);
              if(is.is_open())
              {
                cereal::BinaryInputArchive ar(is);
                ar(gsNamed);
              }
            }
            else
            {
              notLoadedLocations.insert({namedComponent, namedComponentPath.string()});
            }
            container[namedComponent] = gsNamed;
          }
          itNamedComponents++;
        } // end for each file/directory in layer directory
        itLayers++;
      } // for each layers
    }
  } // end for each component


  // TODO: return false on failure
  return true;
}

bool GraphStorageHolder::save(const std::string& dirPath)
{

  // save each edge db separately
  boost::filesystem::path gsParent = boost::filesystem::path(dirPath) / "gs";

  // remove all existing files in the graph storage first, otherwise deleted graphstorages might re-appear
  boost::filesystem::remove_all(gsParent);
  boost::filesystem::create_directories(gsParent);

  for(GraphStorageIt it = container.begin(); it != container.end(); it++)
  {
    boost::this_thread::interruption_point();

    const Component& c = it->first;
    boost::filesystem::path finalPath;
    if(c.name.empty())
    {
      finalPath = gsParent / ComponentTypeHelper::toString(c.type) / c.layer;
    }
    else
    {
      finalPath = gsParent / ComponentTypeHelper::toString(c.type) / c.layer / c.name;
    }
    boost::filesystem::create_directories(finalPath);
    auto outputFile = finalPath / "component.cereal";
    std::ofstream os(outputFile.string(), std::ios::binary);
    cereal::BinaryOutputArchive ar(os);
    ar(it->second);
  }

  // TODO: return false if failed.
  return true;
}


bool GraphStorageHolder::ensureComponentIsLoaded(const Component &c)
{
  auto itGS = container.find(c);
  if(itGS != container.end())
  {
    auto itLocation = notLoadedLocations.find(c);
    if(itLocation != notLoadedLocations.end())
    {
      HL_DEBUG(logger, (boost::format("loading component %1%")
                       % debugComponentString(itLocation->first)).str());
      std::ifstream is(itLocation->second + "/component.cereal");
      if(is.is_open())
      {
        cereal::BinaryInputArchive ar(is);
        ar(itGS->second);
        notLoadedLocations.erase(itLocation);

        return true;
      }
    }
  }
  return false;
}

std::string GraphStorageHolder::debugComponentString(const Component &c)
{
  std::stringstream ss;
  ss << ComponentTypeHelper::toString(c.type) << "|" << c.layer
     << "|" << c.name;
  return ss.str();

}

std::shared_ptr<WriteableGraphStorage> GraphStorageHolder::createWritableGraphStorage(ComponentType ctype, const std::string &layer, const std::string &name)
{
  Component c = {ctype, layer, name == "NULL" ? "" : name};

  // check if there is already an edge DB for this component
  std::map<Component,std::shared_ptr<ReadableGraphStorage>>::const_iterator itDB =
      container.find(c);
  if(itDB != container.end())
  {
    // check if the current implementation is writeable
    std::shared_ptr<WriteableGraphStorage> writable = std::dynamic_pointer_cast<WriteableGraphStorage>(itDB->second);
    if(writable)
    {
      return writable;
    }
  }

  std::shared_ptr<WriteableGraphStorage> gs = std::shared_ptr<WriteableGraphStorage>(new AdjacencyListStorage());
  // register the used implementation
  container[c] = gs;
  return gs;

}

ComponentType GraphStorageHolder::componentTypeFromShortName(std::string shortType)
{
  ComponentType ctype;
  if(shortType == "c")
  {
    ctype = ComponentType::COVERAGE;
  }
  else if(shortType == "d")
  {
    ctype = ComponentType::DOMINANCE;
  }
  else if(shortType == "p")
  {
    ctype = ComponentType::POINTING;
  }
  else if(shortType == "o")
  {
    ctype = ComponentType::ORDERING;
  }
  else
  {
    throw("Unknown component type \"" + shortType + "\"");
  }
  return ctype;
}

