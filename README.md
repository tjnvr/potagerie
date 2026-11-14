## Skills Claude

Les skills du projet sont dans `skills/<nom>/SKILL.md`, versionnées avec le dépôt.

Claude Code les lit dans `.claude/skills/`. Pour qu'il trouve celles du dépôt, crée une fois ce lien depuis la racine :

```
mkdir -p .claude && ln -s ../skills .claude/skills
```
