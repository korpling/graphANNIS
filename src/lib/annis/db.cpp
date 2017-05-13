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

#include "db.h"
#include <annis/annostorage.h>                          // for AnnoStorage
#include <annis/api/graphupdate.h>                      // for UpdateEvent
#include <annis/db.h>                                   // for DB
#include <annis/graphstorage/graphstorage.h>            // for WriteableGrap...
#include <annis/graphstorageregistry.h>                 // for GraphStorageR...
#include <annis/util/helper.h>                          // for Helper
#include <google/btree.h>                               // for btree_iterator
#include <google/btree_container.h>                     // for btree_unique_...
#include <google/btree_map.h>                           // for btree_map
#include <humblelogging/api.h>                          // for HL_INFO, HL_E...
#include <humblelogging/logger.h>                       // for Logger
#include <boost/algorithm/string/predicate.hpp>         // for starts_with
#include <boost/filesystem/operations.hpp>              // for directory_ite...
#include <boost/filesystem/path.hpp>                    // for path, operator/
#include <boost/iterator/iterator_facade.hpp>           // for iterator_faca...
#include <boost/thread/thread.hpp>                      // for interruption_...
#include <cereal/archives/binary.hpp>                   // for BinaryInputAr...
#include <cereal/cereal.hpp>                            // for InputArchive
#include <iostream>                                     // for ifstream, ope...
#include <limits>                                       // for numeric_limits
#include <list>                                         // for list
#include <sstream>
#include "annis/graphstorageholder.h"                   // for GraphStorageH...
#include "annis/stringstorage.h"                        // for StringStorage
#include "annis/types.h"                                // for TextProperty
#include <boost/format.hpp>


HUMBLE_LOGGER(logger, "annis4");

using namespace annis;
using namespace std;

DB::DB()
: edges(strings), currentChangeID(0)
{
  addDefaultStrings();
}

bool DB::load(string dir, bool preloadComponents)
{
  clear();

  boost::filesystem::path dirPath(dir);
  boost::filesystem::path dir2load = dirPath / "current";

  boost::filesystem::path backup = dirPath / "backup";
  bool backupWasLoaded = false;
  if(boost::filesystem::exists(backup) && boost::filesystem::is_directory(backup))
  {
    // load backup instead
    dir2load = backup;
    backupWasLoaded = true;
  }

  std::ifstream is((dir2load / "nodes.cereal").string(), std::ios::binary);
  if(is.is_open())
  {
    cereal::BinaryInputArchive archive(is);
    archive(strings, nodeAnnos);
  }

  bool logfileExists = false;
  // check if we have to apply a log file to get to the last stable snapshot version
  std::ifstream logStream((dir2load / "update_log.cereal").string(), std::ios::binary);
  if(logStream.is_open())
  {
    logfileExists = true;
  }

  // If backup is active or a write log exists, always  a pre-load to get the complete corpus.
  edges.load(dir2load.string(), backupWasLoaded || logfileExists || preloadComponents);

  if(logStream.is_open())
  {
     // apply any outstanding log file updates
     cereal::BinaryInputArchive log(logStream);
     api::GraphUpdate u;
     log(u);
     if(u.getLastConsistentChangeID() > currentChangeID)
     {
       update(u);
     }
  }
  else
  {
    currentChangeID = 0;
  }

  if(backupWasLoaded)
  {
    // save the current corpus under the actual location
    save(dirPath.string());

    // rename backup folder (renaming is atomic and deleting could leave an incomplete backup folder on disk)
    boost::filesystem::path tmpDir =
        boost::filesystem::unique_path(dirPath / "temporary-%%%%-%%%%-%%%%-%%%%");
    boost::filesystem::rename(backup, tmpDir);

    // remove it after renaming it
    boost::filesystem::remove_all(tmpDir);

  }

  // TODO: return false on failure
  return true;
}

bool DB::save(string dir)
{

  // always save to the "current" sub-directory
  boost::filesystem::path dirPath = boost::filesystem::path(dir) / "current";

  boost::filesystem::create_directories(dirPath);

  boost::this_thread::interruption_point();

  std::ofstream os((dirPath / "nodes.cereal").string(), std::ios::binary);
  cereal::BinaryOutputArchive archive( os );
  archive(strings, nodeAnnos);

  boost::this_thread::interruption_point();

  edges.save(dirPath.string());

  boost::this_thread::interruption_point();

  // this is a good time to remove all uncessary data like backups or write logs
  for(auto fileIt = boost::filesystem::directory_iterator(dirPath);
      fileIt != boost::filesystem::directory_iterator(); fileIt++)
  {
    boost::this_thread::interruption_point();
    if(boost::filesystem::is_directory(fileIt->path()))
    {
      if(boost::algorithm::starts_with(fileIt->path().filename().string(), "temporary-"))
      {
        boost::filesystem::remove_all(fileIt->path());
      }
    }
    else if(fileIt->path().filename() == "update_log.cereal")
    {
      boost::filesystem::remove(fileIt->path());
    }
  }

  // TODO: return false on failure
  return true;
}

std::string DB::getNodeDebugName(const nodeid_t &id) const
{
  std::stringstream ss;
  ss << getNodeName(id) << "(" << id << ")";

  return ss.str();
}




void DB::clear()
{
  strings.clear();
  nodeAnnos.clear();
  edges.clear();

  addDefaultStrings();
}

void DB::addDefaultStrings()
{
  annisNamespaceStringID = strings.add(annis_ns);
  annisEmptyStringID = strings.add("");
  annisTokStringID = strings.add(annis_tok);
  annisNodeNameStringID = strings.add(annis_node_name);
  annisNodeTypeID = strings.add(annis_node_type);
}

nodeid_t DB::nextFreeNodeID() const
{
  return nodeAnnos.annotations.empty() ? 0 : (nodeAnnos.annotations.rbegin()->first.id) + 1;
}

void DB::convertComponent(Component c, std::string impl)
{
  map<Component, std::shared_ptr<ReadableGraphStorage>>::const_iterator
      it = edges.container.find(c);
  if(it != edges.container.end())
  {
    std::shared_ptr<ReadableGraphStorage> oldStorage = it->second;

    if(!(oldStorage->getStatistics().valid))
    {
      oldStorage->calculateStatistics(strings);
    }

    std::string currentImpl = edges.registry.getName(oldStorage);
    if(impl == "")
    {
      impl = edges.registry.getOptimizedImpl(c, oldStorage->getStatistics());
    }
    std::shared_ptr<ReadableGraphStorage> newStorage = oldStorage;
    if(currentImpl != impl)
    {
      HL_DEBUG(logger, (boost::format("converting component %1% from %2% to %3%")
                       % edges.debugComponentString(c)
                       % currentImpl
                       % impl).str());

      newStorage = edges.registry.createGraphStorage(impl, strings, c);
      newStorage->copy(*this, *oldStorage);
      edges.container[c] = newStorage;
    }

    // perform index calculations
    std::shared_ptr<WriteableGraphStorage> asWriteableGS = std::dynamic_pointer_cast<WriteableGraphStorage>(newStorage);
    if(asWriteableGS)
    {
      asWriteableGS->calculateIndex();
    }
  }
}

void DB::optimizeAll(const std::map<Component, string>& manualExceptions)
{
  for(const auto& c : getAllComponents())
  {
    edges.ensureComponentIsLoaded(c);
    auto find = manualExceptions.find(c);
    if(find == manualExceptions.end())
    {
      // get the automatic calculated best implementation
      convertComponent(c);
    }
    else
    {
      convertComponent(c, find->second);
    }
  }
}

void DB::ensureAllComponentsLoaded()
{
  for(const auto& c : getAllComponents())
  {
    edges.ensureComponentIsLoaded(c);
  }
}

size_t DB::estimateMemorySize()
{
  return
    nodeAnnos.estimateMemorySize()
      + strings.estimateMemorySize()
      + edges.estimateMemorySize();
}

string DB::info()
{
  stringstream ss;
  ss  << "Number of node annotations: " << nodeAnnos.numberOfAnnotations() << endl
      << "Number of strings in storage: " << strings.size() << endl
      << "Average string length: " << strings.avgLength() << endl
      << "--------------------" << std::endl
      << edges.info() << std::endl;

  return ss.str();
}


std::vector<Component> DB::getDirectConnected(const Edge &edge) const
{
  std::vector<Component> result;
  map<Component, std::shared_ptr<ReadableGraphStorage>>::const_iterator itGS = edges.container.begin();

  while(itGS != edges.container.end())
  {
    std::shared_ptr<ReadableGraphStorage> gs = itGS->second;
    if(gs != NULL)
    {
      if(gs->isConnected(edge))
      {
        result.push_back(itGS->first);
      }
    }
    itGS++;
  }

  return result;
}

std::vector<Component> DB::getAllComponents() const
{
  std::vector<Component> result;
  map<Component, std::shared_ptr<ReadableGraphStorage>>::const_iterator itGS = edges.container.begin();

  while(itGS != edges.container.end())
  {
    result.push_back(itGS->first);
    itGS++;
  }

  return result;
}

vector<Annotation> DB::getEdgeAnnotations(const Component &component,
                                          const Edge &edge)
{
  std::map<Component,std::shared_ptr<ReadableGraphStorage>>::const_iterator it = edges.container.find(component);
  if(it != edges.container.end() && it->second != NULL)
  {
    std::shared_ptr<ReadableGraphStorage> gs = it->second;
    return gs->getEdgeAnnotations(edge);
  }

  return vector<Annotation>();

}

void DB::update(const api::GraphUpdate& u)
{
   for(std::shared_ptr<api::UpdateEvent> change : u.getDiffs())
   {
      if(change->changeID <= u.getLastConsistentChangeID())
      {
         if(std::shared_ptr<api::AddNodeEvent> evt = std::dynamic_pointer_cast<api::AddNodeEvent>(change))
         {
            auto existingNodeID = getNodeID(evt->nodeName);
            // only add node if it does not exist yet
            if(!existingNodeID)
            {
               nodeid_t newNodeID = nextFreeNodeID();
               Annotation newAnnoName =
                  {getNodeNameStringID(), getNamespaceStringID(), strings.add(evt->nodeName)};
               nodeAnnos.addAnnotation(newNodeID, newAnnoName);

               Annotation newAnnoType =
                  {getNodeTypeStringID(), getNamespaceStringID(), strings.add(evt->nodeType)};
               nodeAnnos.addAnnotation(newNodeID, newAnnoType);
            }
         }
         else if(std::shared_ptr<api::DeleteNodeEvent> evt = std::dynamic_pointer_cast<api::DeleteNodeEvent>(change))
         {
            auto existingNodeID = getNodeID(evt->nodeName);
            if(existingNodeID)
            {
               // add all annotations
               std::vector<Annotation> annoList = nodeAnnos.getAnnotations(*existingNodeID);
               for(Annotation anno : annoList)
               {
                  AnnotationKey annoKey = {anno.name, anno.ns};
                  nodeAnnos.deleteAnnotation(*existingNodeID, annoKey);
               }
               // delete all edges pointing to this node either as source or target
               for(Component c : getAllComponents())
               {
                  std::shared_ptr<WriteableGraphStorage> gs =
                    edges.createWritableGraphStorage(c.type, c.layer, c.name);
                  gs->deleteNode(*existingNodeID);
               }

            }
         }
         else if(std::shared_ptr<api::AddNodeLabelEvent> evt = std::dynamic_pointer_cast<api::AddNodeLabelEvent>(change))
         {
            auto existingNodeID = getNodeID(evt->nodeName);
            if(existingNodeID)
            {
              Annotation anno = {strings.add(evt->annoName),
                                 strings.add(evt->annoNs),
                                 strings.add(evt->annoValue)};
              nodeAnnos.addAnnotation(*existingNodeID, anno);
            }
         }
         else if(std::shared_ptr<api::DeleteNodeLabelEvent> evt = std::dynamic_pointer_cast<api::DeleteNodeLabelEvent>(change))
         {
            auto existingNodeID = getNodeID(evt->nodeName);
            if(existingNodeID)
            {
              AnnotationKey annoKey = {strings.add(evt->annoName),
                                       strings.add(evt->annoNs)};
              nodeAnnos.deleteAnnotation(*existingNodeID, annoKey);
            }
         }
         else if(std::shared_ptr<api::AddEdgeEvent> evt = std::dynamic_pointer_cast<api::AddEdgeEvent>(change))
         {
            auto existingSourceID = getNodeID(evt->sourceNode);
            auto existingTargetID = getNodeID(evt->targetNode);
            // only add edge if both nodes already exist
            if(existingSourceID && existingTargetID)
            {
               ComponentType type = ComponentTypeHelper::fromString(evt->componentType);
               if(type < ComponentType::ComponentType_MAX)
               {
                  std::shared_ptr<WriteableGraphStorage> gs =
                    edges.createWritableGraphStorage(type, evt->layer, evt->componentName);
                  gs->addEdge({*existingSourceID, *existingTargetID});
               }
            }
         }
         else if(std::shared_ptr<api::DeleteEdgeEvent> evt = std::dynamic_pointer_cast<api::DeleteEdgeEvent>(change))
         {
            auto existingSourceID = getNodeID(evt->sourceNode);
            auto existingTargetID = getNodeID(evt->targetNode);
            // only delete edge if both nodes actually exist
            if(existingSourceID && existingTargetID)
            {
               ComponentType type = ComponentTypeHelper::fromString(evt->componentType);
               if(type < ComponentType::ComponentType_MAX)
               {
                  std::shared_ptr<WriteableGraphStorage> gs =
                    edges.createWritableGraphStorage(type, evt->layer, evt->componentName);
                  gs->deleteEdge({*existingSourceID, *existingTargetID});
               }
            }
         }
         else if(std::shared_ptr<api::AddEdgeLabelEvent> evt = std::dynamic_pointer_cast<api::AddEdgeLabelEvent>(change))
         {
           auto existingSourceID = getNodeID(evt->sourceNode);
           auto existingTargetID = getNodeID(evt->targetNode);
           // only add label if both nodes already exists
           if(existingSourceID && existingTargetID)
           {
              ComponentType type = ComponentTypeHelper::fromString(evt->componentType);
              if(type < ComponentType::ComponentType_MAX)
              {
                 std::shared_ptr<WriteableGraphStorage> gs =
                   edges.createWritableGraphStorage(type, evt->layer, evt->componentName);

                 // only add label if the edge already exists
                 if(gs->isConnected({*existingSourceID, *existingTargetID}, 1, 1))
                 {
                   Annotation anno = {strings.add(evt->annoName), strings.add(evt->annoNs), strings.add(evt->annoValue)};
                   gs->addEdgeAnnotation({*existingSourceID, *existingTargetID}, anno);
                 }

              }
           }
         }
         else if(std::shared_ptr<api::DeleteEdgeLabelEvent> evt = std::dynamic_pointer_cast<api::DeleteEdgeLabelEvent>(change))
         {
           auto existingSourceID = getNodeID(evt->sourceNode);
           auto existingTargetID = getNodeID(evt->targetNode);
           // only add label if both nodes actually exists
           if(existingSourceID && existingTargetID)
           {
              ComponentType type = ComponentTypeHelper::fromString(evt->componentType);
              if(type < ComponentType::ComponentType_MAX)
              {
                 std::shared_ptr<WriteableGraphStorage> gs =
                   edges.createWritableGraphStorage(type, evt->layer, evt->componentName);

                 // only delete label if the edge actually exists
                 if(gs->isConnected({*existingSourceID, *existingTargetID}, 1, 1))
                 {
                   AnnotationKey annoKey = {strings.add(evt->annoName), strings.add(evt->annoNs)};
                   gs->deleteEdgeAnnotation({*existingSourceID, *existingTargetID}, annoKey);
                 }

              }
           }
         }
         currentChangeID = change->changeID;
      } // end if changeID is behind last consistent
   } // end for each change in update list

}

DB::~DB()
{
}


