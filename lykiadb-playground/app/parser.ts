import {
    Language,
    LanguageSupport,
    defineLanguageFacet,
    syntaxHighlighting,
  } from '@codemirror/language';
  import { Input, NodeType, Parser, PartialParse, Tree } from '@lezer/common';
  import { HighlightStyle } from '@codemirror/language';
  import {styleTags, tags as t, Tag} from "@lezer/highlight"

  const tgs = {
    String: t.string,
    Number: t.number,
    Identifier: t.variableName,
    Boolean: t.bool,
    Keyword: t.link,
    SqlKeyword: t.keyword,
    Symbol: t.operator,
    "Null Undefined": t.null,
  } as Record<string, any>

  export const jsHighlight = styleTags(tgs)

  function convertParsedToLezer(node: any) {
    if (!node.span) {
      return Tree.empty;
    }
    const children = (node.children || []).toSorted((a: any, b: any) => a.span.start - b.span.start);
    return new Tree(
      NodeType.define({
        id: 0,
        name: node.name,
        top: false,
        props: [jsHighlight],
      }),
      children.map(convertParsedToLezer),
      children.map((item) => item.span.start - node.span.start),
      node.span.end - node.span.start
    );
  }

  class LyqlNodeParser extends Parser {
    
    constructor(private parseFn: any) {
      super();
      this.parseFn = parseFn;
    }

    createParse(input: Input): PartialParse {
      const doc = input.read(0, input.length);
      
      return {
        advance: (): Tree | null => {
          let parsed = null;
          try {
            parsed = this.parseFn(doc);
            return new Tree(
              NodeType.define({
                id: 0,
                name: "_root",
                top: false,
                props: [jsHighlight],
              }),
              [convertParsedToLezer(parsed)],
              [parsed.span.start],
              parsed.span.end
            );
          }
          catch (e) {
            console.error(e);
            return Tree.empty;
          }
        },
        parsedPos: input.length,
        stopAt: () => {
        },
        stoppedAt: input.length,
      };
    }
  }
  
  function initLanguage(parseFn: any): Language {
    const facet = defineLanguageFacet();
    const parser = new LyqlNodeParser(parseFn);
    const lyqlLanguage = new Language(facet, parser, [], 'lyql');
    return lyqlLanguage;
  }

  export function lyql(parseFn: any): LanguageSupport {
    return new LanguageSupport(initLanguage(parseFn), [
      syntaxHighlighting(HighlightStyle.define([
        ...Object.entries(tgs).map(([key, tag]) => ({
          tag,
          class: `cm-${(key.toLowerCase().replace(' ', '-'))}`,
        })),
      ])),
    ]);
  }

