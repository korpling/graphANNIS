/*
 * To change this license header, choose License Headers in Project Properties.
 * To change this template file, choose Tools | Templates
 * and open the template in the editor.
 */

/* 
 * File:   Plan.h
 * Author: thomas
 *
 * Created on 1. März 2016, 11:48
 */

#pragma once

#include <memory>
#include <vector>
#include <annis/iterators.h>

namespace annis
{

class Plan
{
public:
  Plan(const std::vector<std::shared_ptr<AnnoIt>>& source);
  
  Plan(const Plan& orig);
  virtual ~Plan();
  
  const std::vector<std::shared_ptr<AnnoIt>>& getSource() { return source;}
  double getCost();
  
private:
  std::vector<std::shared_ptr<AnnoIt>> source;
  
  double cost;
  
};

} // end namespace annis