/* 
 * File:   nodeannostorage.cpp
 * Author: thomas
 * 
 * Created on 14. Januar 2016, 13:53
 */

#include <annis/nodeannostorage.h>

#include <annis/stringstorage.h>

#include <re2/re2.h>

#include <fstream>
#include <random>

#include <cereal/archives/binary.hpp>
#include <cereal/types/map.hpp>
#include <cereal/types/set.hpp>

#include "annis/annosearch/annotationsearch.h"

#include <annis/util/size_estimator.h>

using namespace annis;


NodeAnnoStorage::NodeAnnoStorage(StringStorage& strings)
: AnnotationStatisticHolder(strings), strings(strings)
{
}

void NodeAnnoStorage::addNodeAnnotationBulk(std::list<std::pair<NodeAnnotationKey, uint32_t> > annos)
{

  annos.sort();
  nodeAnnotations.insert(annos.begin(), annos.end());

  std::list<std::pair<Annotation, nodeid_t>> inverseAnnos;
  std::list<AnnotationKey> annoKeyList;

  for(const auto& entry : annos)
  {
    const NodeAnnotationKey& key = entry.first;
    inverseAnnos.push_back(std::pair<Annotation, nodeid_t>({key.anno_name, key.anno_ns, entry.second}, key.node));
    annoKeyList.push_back({key.anno_name, key.anno_ns});
  }

  inverseAnnos.sort();

  inverseNodeAnnotations.insert(inverseAnnos.begin(), inverseAnnos.end());

  for(auto annoKey : annoKeyList)
  {
    btree::btree_map<AnnotationKey, size_t>::iterator itKey = nodeAnnoKeys.find(annoKey);
    if(itKey == nodeAnnoKeys.end())
    {
       nodeAnnoKeys.insert({annoKey, 1});
    }
    else
    {
       itKey->second++;
    }
  }

}

void NodeAnnoStorage::clear()
{
  AnnotationStatisticHolder::clear();

  nodeAnnotations.clear();
  inverseNodeAnnotations.clear();
  nodeAnnoKeys.clear();
  
}

size_t NodeAnnoStorage::estimateMemorySize()
{
  return
      size_estimation::element_size(nodeAnnotations)
      + size_estimation::element_size(inverseNodeAnnotations)
      + size_estimation::element_size(nodeAnnoKeys)
      + AnnotationStatisticHolder::estimateMemorySize();
}



NodeAnnoStorage::~NodeAnnoStorage()
{
}

std::vector<Annotation> NodeAnnoStorage::getAnnotationRange(Annotation minAnno, Annotation maxAnno)
{
  auto itUpperBound = inverseNodeAnnotations.upper_bound(maxAnno);
  std::vector<Annotation> annos;
  for(auto it=inverseNodeAnnotations.lower_bound(minAnno); it != itUpperBound; it++)
  {
    annos.push_back(it->first);
  }
  return std::move(annos);
}

