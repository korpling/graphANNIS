#include "linearedgedb.h"

using namespace annis;
using namespace std;

LinearEdgeDB::LinearEdgeDB()
{
}

void LinearEdgeDB::addEdge(const Edge &edge)
{
  throw("Not implemented yet");
}

void LinearEdgeDB::addEdgeAnnotation(const Edge &edge, const Annotation &anno)
{
  throw("Not implemented yet");
}

void LinearEdgeDB::clear()
{
  node2pos.clear();
  pos2node.clear();
}

bool LinearEdgeDB::isConnected(const Edge &edge, unsigned int minDistance, unsigned int maxDistance) const
{
  throw("Not implemented yet");
}

EdgeIterator *LinearEdgeDB::findConnected(nodeid_t sourceNode, unsigned int minDistance, unsigned int maxDistance) const
{
  throw("Not implemented yet");
}

std::vector<Annotation> LinearEdgeDB::getEdgeAnnotations(const Edge &edge) const
{
  throw("Not implemented yet");
}

std::vector<nodeid_t> LinearEdgeDB::getOutgoingEdges(nodeid_t sourceNode) const
{
  throw("Not implemented yet");
}

std::uint32_t LinearEdgeDB::numberOfEdgeAnnotations() const
{
  throw("Not implemented yet");
}

std::uint32_t LinearEdgeDB::numberOfEdges() const
{
  throw("Not implemented yet");
}

bool LinearEdgeDB::save(string dirPath)
{
  throw("Not implemented yet");
}

bool LinearEdgeDB::load(string dirPath)
{
  throw("Not implemented yet");
}

LinearEdgeDB::~LinearEdgeDB()
{

}
