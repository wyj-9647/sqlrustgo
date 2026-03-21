# SQLRustGo 用户故事

## US-001: 执行 SELECT 查询
**作为** 用户  
**我想要** 执行 SELECT 查询语句  
**以便于** 从数据库中检索数据  

**验收标准**：
- 支持 SELECT * FROM users
- 支持 WHERE 条件
- 支持 ORDER BY 排序

## US-002: 执行 INSERT 操作
**作为** 用户  
**我想要** 向表中插入数据  
**以便于** 添加新记录  

**验收标准**：
- 支持 INSERT INTO users VALUES (...)
- 支持指定列插入

## US-003: 执行 UPDATE 操作
**作为** 用户  
**我想要** 更新表中的数据  
**以便于** 修改已有记录  

**验收标准**：
- 支持 UPDATE users SET ... WHERE ...
- 支持多列更新

## US-004: 执行 DELETE 操作
**作为** 用户  
**我想要** 删除表中的数据  
**以便于** 移除不需要的记录  

**验收标准**：
- 支持 DELETE FROM users WHERE ...
- 支持删除所有数据（不带 WHERE）