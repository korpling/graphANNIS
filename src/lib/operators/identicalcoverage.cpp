/* 
 * File:   IdenticalCoverage.cpp
 * Author: thomas
 * 
 * Created on 8. Januar 2016, 13:58
 */

#include "identicalcoverage.h"

using namespace annis;

IdenticalCoverage::IdenticalCoverage()
{
}

bool IdenticalCoverage::filter(const Match& lhs, const Match& rhs)
{
  return false;
}

std::unique_ptr<AnnoIt> IdenticalCoverage::retrieveMatches(const Match& lhs)
{
  return std::unique_ptr<AnnoIt>();
}



IdenticalCoverage::~IdenticalCoverage()
{
}

