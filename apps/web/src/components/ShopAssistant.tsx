"use client";

import { useEffect, useRef, useState } from "react";
import { assistantActions, buildWorkerHandoff, responseForAction, type AssistantActionId } from "../lib/assistant";

interface ShopAssistantProps {
  onRent: () => void;
  onList: () => void;
  onWorkspace: () => void;
  onDelivery: () => void;
}

interface ChatMessage { id:number; author:"BSR Assistant"|"You"; text:string }

const greeting = "Hi! I’m the BSR Assistant. I can help you rent, sell, book a workspace, arrange delivery, or prepare a message for our team.";

export function ShopAssistant({ onRent, onList, onWorkspace, onDelivery }: ShopAssistantProps) {
  const [open,setOpen]=useState(false);
  const [messages,setMessages]=useState<ChatMessage[]>([{id:1,author:"BSR Assistant",text:greeting}]);
  const [workerMode,setWorkerMode]=useState(false);
  const [workerMessage,setWorkerMessage]=useState("");
  const [handoffStatus,setHandoffStatus]=useState("");
  const nextId=useRef(2);
  const timer=useRef<ReturnType<typeof setTimeout>|null>(null);
  const supportDestination=process.env.NEXT_PUBLIC_SUPPORT_EMAIL ?? "";

  useEffect(()=>{
    try {
      if (sessionStorage.getItem("bsr-assistant-greeted")) return;
      timer.current=setTimeout(()=>{
        setOpen(true);
        sessionStorage.setItem("bsr-assistant-greeted","true");
      },1600);
    } catch { timer.current=setTimeout(()=>setOpen(true),1600); }
    return ()=>{ if(timer.current)clearTimeout(timer.current); };
  },[]);

  useEffect(()=>{
    if(!open)return;
    const closeOnEscape=(event:KeyboardEvent)=>{ if(event.key==="Escape")setOpen(false); };
    document.addEventListener("keydown",closeOnEscape);
    return ()=>document.removeEventListener("keydown",closeOnEscape);
  },[open]);

  const markGreeted=()=>{ try{sessionStorage.setItem("bsr-assistant-greeted","true");}catch{} };
  const dismiss=()=>{ if(timer.current)clearTimeout(timer.current);markGreeted();setOpen(false); };
  const launch=()=>{ markGreeted();setOpen(true); };

  const choose=(id:AssistantActionId)=>{
    const label=assistantActions.find(action=>action.id===id)?.label ?? id;
    setMessages(current=>[...current,{id:nextId.current++,author:"You",text:label},{id:nextId.current++,author:"BSR Assistant",text:responseForAction(id)}]);
    setWorkerMode(id==="worker");
    setHandoffStatus("");
    if(id==="rent")onRent();
    if(id==="list")onList();
    if(id==="workspace")onWorkspace();
    if(id==="delivery")onDelivery();
  };

  const prepareHandoff=async()=>{
    const clean=workerMessage.trim();
    if(!clean){setHandoffStatus("Please describe what you need help with first.");return;}
    const handoff=buildWorkerHandoff(clean,supportDestination);
    if(handoff.mode==="email"){
      setHandoffStatus("Your email app will open with a prepared message. You choose whether to send it.");
      window.location.href=handoff.href;
      return;
    }
    try {
      await navigator.clipboard.writeText(handoff.message);
      setHandoffStatus("Message copied. Share it with a BSR team member; it has not been sent automatically.");
    } catch {
      setHandoffStatus("Copy the message above and share it with a BSR team member. It has not been sent.");
    }
  };

  return <>
    {!open&&<button className="assistant-launcher" onClick={launch} aria-label="Open BSR shopping assistant"><span>✦</span><b>Need help?</b></button>}
    {open&&<aside className="assistant-panel" role="dialog" aria-labelledby="assistant-title">
      <header><div><span className="assistant-mark">✦</span><div><h2 id="assistant-title">BSR Assistant</h2><small>Automated help · no sensitive information</small></div></div><button onClick={dismiss} aria-label="Close assistant">×</button></header>
      <div className="assistant-messages" aria-live="polite">
        {messages.map(message=><div key={message.id} className={`assistant-message ${message.author==="You"?"from-user":""}`}><b>{message.author}</b><p>{message.text}</p></div>)}
      </div>
      <div className="assistant-actions" aria-label="Quick help topics">
        {assistantActions.map(action=><button key={action.id} onClick={()=>choose(action.id)}>{action.label}</button>)}
      </div>
      {workerMode&&<div className="worker-handoff"><label htmlFor="worker-message">Message for the BSR team</label><textarea id="worker-message" value={workerMessage} onChange={event=>setWorkerMessage(event.target.value)} placeholder="Example: I need help arranging delivery for a PS5 rental."/><small>Do not include card numbers, ID documents, passwords, or an exact address.</small><button className="primary" onClick={prepareHandoff}>{supportDestination?"Open prepared email":"Copy message for a worker"}</button>{handoffStatus&&<p className="handoff-status" role="status">{handoffStatus}</p>}</div>}
      <footer>Demo assistant · A worker only replies through a configured team channel.</footer>
    </aside>}
  </>;
}
