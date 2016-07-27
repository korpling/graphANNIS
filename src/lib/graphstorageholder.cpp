#include <annis/graphstorageholder.h>

#include <annis/graphstorage/adjacencyliststorage.h>

#include <sstream>

#include <boost/filesystem.hpp>
#include <boost/format.hpp>
#include <humblelogging/api.h>

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

std::weak_ptr<const ReadableGraphStorage> GraphStorageHolder::getGraphStorage(const Component &component)
{
  std::map<Component, std::shared_ptr<ReadableGraphStorage>>::const_iterator itGS = container.find(component);
  if(itGS != container.end())
  {
    ensureComponentIsLoaded(itGS->first);
    return itGS->second;
  }
  return std::weak_ptr<const ReadableGraphStorage>();
}

std::weak_ptr<const ReadableGraphStorage> GraphStorageHolder::getGraphStorage(ComponentType type, const std::string &layer, const std::string &name)
{
  Component c = {type, layer, name};
  return getGraphStorage(c);
}

std::vector<std::weak_ptr<const ReadableGraphStorage> > GraphStorageHolder::getGraphStorage(ComponentType type, const std::string &name)
{
  std::vector<std::weak_ptr<const ReadableGraphStorage> > result;

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

std::vector<std::weak_ptr<const ReadableGraphStorage> > GraphStorageHolder::getGraphStorage(ComponentType type)
{
  std::vector<std::weak_ptr<const ReadableGraphStorage>> result;

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
    result += e.second->estimateMemorySize();
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


    ss << "Component " << debugComponentString(c) << ": " << gs->numberOfEdges() << " edges and "
       << gs->numberOfEdgeAnnotations() << " annotations" << std::endl;


    std::string implName = registry.getName(gs);
    if(!implName.empty())
    {
      ss << "implementation: " << implName << std::endl;
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

        std::string implName = getImplNameForPath(layerPath.string());

        if(!implName.empty())
        {
          // try to load the component with the empty name
          Component emptyNameComponent = {(ComponentType) componentType,
              layerPath.filename().string(), ""};

          std::shared_ptr<ReadableGraphStorage> gsEmptyName = registry.createGraphStorage(implName, strings, emptyNameComponent);

          if(preloadComponents)
          {
            HL_DEBUG(logger, (boost::format("loading component %1%")
                             % debugComponentString(emptyNameComponent)).str());
            gsEmptyName->load(layerPath.string());
          }
          else
          {
            notLoadedLocations.insert({emptyNameComponent, layerPath.string()});
          }
          container.insert({emptyNameComponent,gsEmptyName});
        }

        // also load all named components
        boost::filesystem::directory_iterator itNamedComponents(layerPath);
        while(itNamedComponents != fileEndIt)
        {
          const boost::filesystem::path namedComponentPath = *itNamedComponents;
          if(boost::filesystem::is_directory(namedComponentPath))
          {
            // try to load the named component
            implName = getImplNameForPath(namedComponentPath.string());
            Component namedComponent = {(ComponentType) componentType,
                                                           layerPath.filename().string(),
                                                           namedComponentPath.filename().string()
                                       };


            std::shared_ptr<ReadableGraphStorage> gsNamed = registry.createGraphStorage(implName, strings, namedComponent);
            if(preloadComponents)
            {
              HL_DEBUG(logger, (boost::format("loading component %1%")
                               % debugComponentString(namedComponent)).str());
              gsNamed->load(namedComponentPath.string());
            }
            else
            {
              notLoadedLocations.insert({namedComponent, namedComponentPath.string()});
            }
            container.insert({namedComponent,gsNamed});
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
  std::ofstream out;

  // save each edge db separately
  std::string gsParent = dirPath + "/gs";
  for(GraphStorageIt it = container.begin(); it != container.end(); it++)
  {
    const Component& c = it->first;
    std::string finalPath;
    if(c.name.empty())
    {
      finalPath = gsParent + "/" + ComponentTypeHelper::toString(c.type) + "/" + c.layer;
    }
    else
    {
      finalPath = gsParent + "/" + ComponentTypeHelper::toString(c.type) + "/" + c.layer + "/" + c.name;
    }
    boost::filesystem::create_directories(finalPath);
    it->second->save(finalPath);
    // put an identification file to the output directory that contains the name of the graph storage implementation
    out.open(finalPath + "/implementation.cfg");
    out << registry.getName(it->second) << std::endl;
    out.close();
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
      itGS->second->load(itLocation->second);
      notLoadedLocations.erase(itLocation);

      return true;
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

std::string GraphStorageHolder::getImplNameForPath(std::string directory)
{
  std::string result = "";
  std::ifstream in(directory + "/implementation.cfg");
  if(in.is_open())
  {
    in >> result;
  }
  in.close();
  return result;
}

std::shared_ptr<ReadableGraphStorage> GraphStorageHolder::createGSForComponent(const std::string &shortType, const std::string &layer, const std::string &name)
{
  // fill the component variable
  ComponentType ctype = componentTypeFromShortName(shortType);
  return createGSForComponent(ctype, layer, name);

}

std::shared_ptr<ReadableGraphStorage> GraphStorageHolder::createGSForComponent(ComponentType ctype, const std::string &layer, const std::string &name)
{
  Component c = {ctype, layer, name};

  // check if there is already an edge DB for this component
  std::map<Component,std::shared_ptr<ReadableGraphStorage>>::const_iterator itDB =
      container.find(c);
  if(itDB == container.end())
  {

    std::shared_ptr<ReadableGraphStorage> gs = registry.createGraphStorage(strings, c, gs->getStatistics());

    // register the used implementation
    container.insert({c,gs});
    return gs;
  }
  else
  {
    return itDB->second;
  }
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

  std::shared_ptr<WriteableGraphStorage> gs = std::shared_ptr<WriteableGraphStorage>(new AdjacencyListStorage(strings, c));
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

