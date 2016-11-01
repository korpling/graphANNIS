#include "graph.h"

#include <boost/thread/shared_lock_guard.hpp>

using namespace annis::api;

Graph::Graph()
{
  db = std::make_shared<DB>();
}

void Graph::addNode(std::string name)
{
  if(db)
  {
    boost::shared_lock_guard<DB> lock(*db);
    nodeid_t id = db->nodeAnnos.nextFreeID();
    Annotation nodeAnno = {db->strings.add(annis_node_name),
                           db->strings.add(annis_ns), db->strings.add(name)};
    db->nodeAnnos.addNodeAnnotation(id, nodeAnno);
  }
}
