#include "corpusstorage.h"

#include <boost/thread/lock_guard.hpp>
#include <boost/thread/shared_lock_guard.hpp>

#include <annis/db.h>

using namespace annis;
using namespace annis::api;

CorpusStorage::CorpusStorage(std::string databaseDir)
  : databaseDir(databaseDir)
{
  cache = std::unique_ptr<DBCache>(new DBCache());
}

CorpusStorage::~CorpusStorage() {}

long long CorpusStorage::count(std::vector<std::string> corpora, std::string queryAsJSON)
{
  long long result = 0;

  // sort corpora by their name
  std::sort(corpora.begin(), corpora.end());

  for(const std::string& c : corpora)
  {
    std::shared_ptr<DB> db = cache->get(databaseDir + "/" + c, false);

    if(db)
    {
      boost::shared_lock_guard<DB> lock(*db);

      std::stringstream ss;
      ss << queryAsJSON;
      std::shared_ptr<annis::Query> q = annis::JSONQueryParser::parse(*db, db->edges, ss);
      while(q->next())
      {
        result++;
      }
    }
  }
  return result;
}

CorpusStorage::CountResult CorpusStorage::countExtra(std::vector<std::string> corpora, std::string queryAsJSON)
{
  CountResult result = {0,0};

  std::set<std::uint32_t> documents;

  // sort corpora by their name
  std::sort(corpora.begin(), corpora.end());

  for(const std::string& c : corpora)
  {
    std::shared_ptr<DB> db = cache->get(databaseDir + "/" + c, false);

    if(db)
    {
      boost::shared_lock_guard<DB> lock(*db);

      std::stringstream ss;
      ss << queryAsJSON;
      std::shared_ptr<annis::Query> q = annis::JSONQueryParser::parse(*db, db->edges, ss);
      while(q->next())
      {
        result.matchCount++;
        const std::vector<Match>& m = q->getCurrent();
        if(!m.empty())
        {
          const Match& n  = m[0];
          std::pair<bool, Annotation> anno = db->nodeAnnos.getNodeAnnotation(n.node, annis_ns, "document");
          if(anno.first)
          {
            documents.insert(anno.second.val);
          }
        }
      }
    }
  }

  result.documentCount = documents.size();
  return result;
}

std::vector<std::string> CorpusStorage::find(std::vector<std::string> corpora, std::string queryAsJSON, long long offset, long long limit)
{
  std::vector<std::string> result;

  long long counter = 0;

  // sort corpora by their name
  std::sort(corpora.begin(), corpora.end());

  for(const std::string& c : corpora)
  {
    std::shared_ptr<DB> db = cache->get(databaseDir + "/" + c, false);

    if(db)
    {
      boost::shared_lock_guard<DB> lock(*db);

      std::stringstream ss;
      ss << queryAsJSON;
      std::shared_ptr<annis::Query> q = annis::JSONQueryParser::parse(*db, db->edges, ss);
      while(counter < (offset + limit) && q->next())
      {
        if(counter >= offset)
        {
          const std::vector<Match>& m = q->getCurrent();
          std::stringstream matchDesc;
          for(size_t i = 0; i < m.size(); i++)
          {
            const Match& n = m[i];

            if(n.anno.ns != 0 && n.anno.name != 0
               && n.anno.ns != db->getNamespaceStringID() && n.anno.name != db->getNodeNameStringID())
            {
              matchDesc << db->strings.str(n.anno.ns)
                << "::" << db->strings.str(n.anno.name)
                << "::";
            }

            matchDesc << "salt:/" << c << "/";
            matchDesc << db->getNodeDocument(n.node) << "/#" << db->getNodeName(n.node);

            if(i < m.size()-1)
            {
             matchDesc << " ";
            }
          }
          result.push_back(matchDesc.str());
        } // end if result in offset-limit range
        counter++;
      }
    }
  }

  return result;
}

void CorpusStorage::applyUpdate(std::string corpus, const GraphUpdate &update)
{
   // we have to make sure that the corpus is fully loaded (with all components) before we can apply the update.
   std::shared_ptr<DB> db = cache->get(databaseDir + "/" + corpus, true);

   if(db)
   {
      boost::shared_lock_guard<DB> lock(*db);

      StringStorage& strings = db->strings;
      NodeAnnoStorage& nodeAnnos = db->nodeAnnos;

      for(const auto& change : update.diffs)
      {
         switch(change.type)
         {
            case GraphUpdate::add_node:
               {
                  auto existingNodeID = db->getNodeID(change.arg0);
                  // only add node if it does not exist yet
                  if(!existingNodeID)
                  {
                     nodeid_t newNodeID = nodeAnnos.nextFreeID();
                     Annotation newAnno =
                        {db->getNamespaceStringID(), db->getNodeNameStringID(), strings.add(change.arg0)};
                     nodeAnnos.addNodeAnnotation(newNodeID, newAnno);
                  }
               }
               break;
            case GraphUpdate::delete_node:
               {
                  auto existingNodeID = db->getNodeID(change.arg0);
                  if(existingNodeID)
                  {
                     // add all annotations
                     std::list<Annotation> annoList = db->nodeAnnos.getNodeAnnotationsByID(*existingNodeID);
                     for(Annotation anno : annoList)
                     {
                        AnnotationKey annoKey = {anno.name, anno.ns};
                        db->nodeAnnos.deleteNodeAnotation(*existingNodeID, annoKey);
                     }
                     // delete all edges pointing to this node either as source or target
                     for(Component c : db->getAllComponents())
                     {
                        std::shared_ptr<WriteableGraphStorage> gs =
                          db->edges.createWritableGraphStorage(c.type, c.layer, c.name);
                        gs->deleteNode(*existingNodeID);
                     }

                  }
               }
               break;
            case GraphUpdate::add_label:
               {

               }
               break;
            case GraphUpdate::delete_label:
               {

               }
               break;
            default:
               throw "Unknown change type";
         }

         // TODO: apply each change
      }
      // TODO: if successfull write log
      // TODO: start background task to write the complete new version without log on the disk

      try {

      } catch (...)
      {
         // TODO: on exception reload the original corpus from disk

      }

   }
}
