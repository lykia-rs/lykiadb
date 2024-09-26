"use client"

import * as React from "react"
import {
  CodeIcon,
} from "lucide-react"

import { cn } from "@/lib/utils"

import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable"
import { Separator } from "@/components/ui/separator"
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/ui/tabs"
import { TooltipProvider } from "@/components/ui/tooltip"
import { Nav } from "./nav"

import CodeMirror from '@uiw/react-codemirror';
import JsonView from '@uiw/react-json-view';
import init, { parse, tokenize } from "../pkg/index";
import { lyql } from "./parser";
import "./lyqlSyntax.scss";

await init();

const MainView = ({
  defaultLayout = [20, 32, 48],
  defaultCollapsed = false,
  navCollapsedSize = 4
}) => {

  const [isCollapsed, setIsCollapsed] = React.useState(defaultCollapsed)

  const [code, setCode] = React.useState(`SELECT { 'name': user.name } from users;`);

  const [ast, setAst] = React.useState({});


  function updateCode(code: string) {
    setCode(code)
    try {
      const parsed = parse(code);
      if (parsed) setAst(parsed);
    }
    catch (e) {
      console.error(e);
    }
  }

  return (
    <TooltipProvider delayDuration={0}>
      <ResizablePanelGroup
        direction="horizontal"
        onLayout={(sizes: number[]) => {
          document.cookie = `react-resizable-panels:layout:main=${JSON.stringify(
            sizes
          )}`
        }}
        className="h-full max-h-[800px] items-stretch"
      >
        <ResizablePanel
          defaultSize={defaultLayout[0]}
          collapsedSize={navCollapsedSize}
          collapsible={true}
          minSize={15}
          maxSize={20}
          onCollapse={() => {
            setIsCollapsed(true)
            document.cookie = `react-resizable-panels:collapsed=${JSON.stringify(
              true
            )}`
          }}
          onResize={() => {
            setIsCollapsed(false)
            document.cookie = `react-resizable-panels:collapsed=${JSON.stringify(
              false
            )}`
          }}
          className={cn(
            isCollapsed &&
              "min-w-[50px] transition-all duration-300 ease-in-out"
          )}
        >
          <div
            className={cn(
              "flex h-[52px] items-center justify-center",
              isCollapsed ? "h-[52px]" : "px-2"
            )}
          >
          </div>
          <Separator />
          <Nav
            isCollapsed={isCollapsed}
            links={[
              {
                title: "Editor",
                label: "",
                icon: CodeIcon,
                variant: "default",
              },
            ]}
          />
        </ResizablePanel>
        <ResizableHandle withHandle />
        <ResizablePanel defaultSize={defaultLayout[1]} minSize={30}>
          <Tabs defaultValue="code">
            <div className="flex items-center px-4 py-3">
              <h1 className="text-xl font-bold">Code</h1>
            </div>
            <Separator />
            <TabsContent value="code" className="m-0 p-4">
              <CodeMirror
                value={code}
                extensions={[lyql(tokenize)]} 
                onChange={(value: string) => updateCode(value)} 
              />
            </TabsContent>
          </Tabs>
        </ResizablePanel>
        <ResizableHandle withHandle />
        <ResizablePanel defaultSize={defaultLayout[2]} minSize={30}>
          <Tabs defaultValue="ast">
            <div className="flex items-center px-4 py-2">
              <h1 className="text-xl font-bold">Inspect</h1>
              <TabsList className="ml-auto">
                <TabsTrigger
                  value="ast"
                  className="text-zinc-600 dark:text-zinc-200"
                >
                  AST
                </TabsTrigger>
                <TabsTrigger
                  value="plan"
                  className="text-zinc-600 dark:text-zinc-200"
                >
                  Plan
                </TabsTrigger>
              </TabsList>
            </div>
            <Separator />
            <TabsContent value="ast" className="m-0 p-4">
              <JsonView value={ast} />
            </TabsContent>
            <TabsContent value="plan" className="m-0">
            </TabsContent>
          </Tabs>
        </ResizablePanel>
      </ResizablePanelGroup>
    </TooltipProvider>
  )
}

export default MainView