import React from "react";
import { api, uploadFile } from "../api";
import type {
  Batch,
  CreateBatchInput,
  Paginated,
  Region,
  School,
  SchoolClassPlan,
  SchoolDeleteImpact,
  SchoolProfileDraft,
  SchoolRegionHistory,
  SchoolProgramDashboard,
  SipMasterImportPreview,
  SipMasterImportResult,
  Student,
  LectureModel,
  UpdateBatchInput,
} from "../types";

// ── Utility ─────────────────────────────────────────────────────────────────

function schoolToProfileDraft(school: School): SchoolProfileDraft {
  return {
    name: school.name,
    region_id: school.region_id,
    region_name: school.region_name,
    program_model: school.program_model,
    distance_classification: school.distance_classification,
    sip_academic_owner_role: school.sip_academic_owner_role,
    sip_academic_owner_name: school.sip_academic_owner_name,
    sip_academic_owner_mobile: school.sip_academic_owner_mobile,
    sip_academic_owner_email: school.sip_academic_owner_email,
    center_head_name: school.center_head_name,
    center_head_mobile: school.center_head_mobile,
    center_head_email: school.center_head_email,
    principal_name: school.principal_name,
    principal_mobile: school.principal_mobile,
    principal_email: school.principal_email,
    school_spoc_name: school.school_spoc_name,
    school_spoc_mobile: school.school_spoc_mobile,
    school_spoc_email: school.school_spoc_email,
    central_academic_spoc_name: school.central_academic_spoc_name,
    central_academic_spoc_mobile: school.central_academic_spoc_mobile,
    central_academic_spoc_email: school.central_academic_spoc_email,
    central_business_spoc_name: school.central_business_spoc_name,
    central_business_spoc_mobile: school.central_business_spoc_mobile,
    central_business_spoc_email: school.central_business_spoc_email,
    bh_name: school.bh_name,
    bh_mobile: school.bh_mobile,
    bh_email: school.bh_email,
    aom_name: school.aom_name,
    aom_mobile: school.aom_mobile,
    aom_email: school.aom_email,
    mapped_vp_center: school.mapped_vp_center,
    vp_tagging: school.vp_tagging,
  };
}

// ── Interfaces ──────────────────────────────────────────────────────────────

export interface UseMasterDataStateOptions {
  onError: (msg: string) => void;
  onNotice: (msg: string) => void;
  onLoadAuditLog: () => Promise<void>;
}

export interface UseMasterDataStateReturn {
  // State
  schools: School[];
  droppedSchools: School[];
  regions: Region[];
  lectureModels: LectureModel[];
  classPlans: SchoolClassPlan[];
  students: Student[];
  batches: Batch[];
  schoolRegionHistory: SchoolRegionHistory[];
  programDashboard: SchoolProgramDashboard | null;
  sipImportReview: { sourcePath: string; preview: SipMasterImportPreview } | null;
  studentTotalCount: number;
  studentPage: number;
  studentPageSize: number;
  studentSearch: string;

  // Setters
  setSchools: React.Dispatch<React.SetStateAction<School[]>>;
  setDroppedSchools: React.Dispatch<React.SetStateAction<School[]>>;
  setRegions: React.Dispatch<React.SetStateAction<Region[]>>;
  setLectureModels: React.Dispatch<React.SetStateAction<LectureModel[]>>;
  setClassPlans: React.Dispatch<React.SetStateAction<SchoolClassPlan[]>>;
  setStudents: React.Dispatch<React.SetStateAction<Student[]>>;
  setStudentSearch: React.Dispatch<React.SetStateAction<string>>;
  setBatches: React.Dispatch<React.SetStateAction<Batch[]>>;
  setSchoolRegionHistory: React.Dispatch<React.SetStateAction<SchoolRegionHistory[]>>;
  setProgramDashboard: React.Dispatch<React.SetStateAction<SchoolProgramDashboard | null>>;
  setSipImportReview: React.Dispatch<React.SetStateAction<{ sourcePath: string; preview: SipMasterImportPreview } | null>>;

  // Loaders
  loadSchools: () => Promise<void>;
  loadDroppedSchools: () => Promise<void>;
  loadSchoolRegionHistory: () => Promise<void>;
  loadRegions: () => Promise<void>;
  loadLectureModels: () => Promise<void>;
  loadClassPlans: (schoolId?: number) => Promise<void>;
  loadProgramDashboard: () => Promise<void>;
  loadStudents: (schoolId?: number, page?: number, search?: string) => Promise<void>;
  loadBatches: (schoolId?: number) => Promise<void>;
  loadSchoolDeleteImpact: (id: number) => Promise<SchoolDeleteImpact | null>;

  // Mutations
  createSchool: (input: SchoolProfileDraft) => Promise<void>;
  dropSchool: (id: number, reason: string) => Promise<void>;
  deleteSchool: (id: number) => Promise<void>;
  restoreSchool: (id: number) => Promise<void>;
  saveRegion: (input: {
    id?: number;
    name: string;
    regional_academic_head_name: string;
    regional_academic_head_mobile: string;
    regional_academic_head_email: string;
    regional_business_head_name: string;
    regional_business_head_mobile: string;
    regional_business_head_email: string;
  }) => Promise<void>;
  deleteRegion: (id: number) => Promise<void>;
  remapAndDeleteRegion: (
    regionId: number,
    mappings: Array<{
      school_id: number;
      target_region_id?: number;
      new_region_name?: string;
    }>,
  ) => Promise<void>;
  createLectureModel: (input: {
    name: string;
    days_per_week: number;
    lectures_per_day: number;
  }) => Promise<void>;
  saveSchoolClassPlan: (input: {
    school_id: number;
    grade_level: string;
    track: string;
    lecture_model_id: number;
    batch_pattern: string;
    aop_admissions: number;
    registrations: number;
    actual_admissions: number;
  }) => Promise<void>;
  createBatch: (input: CreateBatchInput) => Promise<void>;
  updateBatch: (input: UpdateBatchInput) => Promise<void>;
  archiveBatch: (id: number) => Promise<void>;
  createStudent: (input: Record<string, unknown>) => Promise<void>;
  updateStudent: (input: Record<string, unknown>) => Promise<void>;
  deleteStudent: (id: number) => Promise<void>;

  // CSV imports
  importSchoolsCsv: () => void;
  importStudentsCsv: (schoolId: number) => void;
  importSipMaster: () => void;
  confirmSipMasterImport: (
    conflictAction: "skip_existing" | "update_existing",
  ) => Promise<void>;
  cancelSipMasterImport: () => void;

  // Export
  exportSipMasterExcel: () => Promise<void>;
}

// ── Hook ────────────────────────────────────────────────────────────────────

export function useMasterDataState(options: UseMasterDataStateOptions): UseMasterDataStateReturn {
  const { onError, onNotice, onLoadAuditLog } = options;

  // ── State ─────────────────────────────────────────────────────────────────
  const [schools, setSchools] = React.useState<School[]>([]);
  const [droppedSchools, setDroppedSchools] = React.useState<School[]>([]);
  const [schoolRegionHistory, setSchoolRegionHistory] = React.useState<SchoolRegionHistory[]>([]);
  const [regions, setRegions] = React.useState<Region[]>([]);
  const [lectureModels, setLectureModels] = React.useState<LectureModel[]>([]);
  const [classPlans, setClassPlans] = React.useState<SchoolClassPlan[]>([]);
  const [programDashboard, setProgramDashboard] = React.useState<SchoolProgramDashboard | null>(null);
  const [students, setStudents] = React.useState<Student[]>([]);
  const [studentTotalCount, setStudentTotalCount] = React.useState(0);
  const [studentPage, setStudentPage] = React.useState(1);
  const [studentPageSize] = React.useState(100);
  const [studentSearch, setStudentSearch] = React.useState("");
  const [batches, setBatches] = React.useState<Batch[]>([]);
  const [sipImportReview, setSipImportReview] = React.useState<{
    sourcePath: string;
    preview: SipMasterImportPreview;
  } | null>(null);

  const pendingSipFileRef = React.useRef<File | null>(null);

  // ── Loaders ───────────────────────────────────────────────────────────────

  const loadSchools = React.useCallback(async () => {
    try {
      setSchools(await api<School[]>("list_schools"));
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadDroppedSchools = React.useCallback(async () => {
    try {
      setDroppedSchools(await api<School[]>("list_dropped_schools"));
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadSchoolRegionHistory = React.useCallback(async () => {
    try {
      setSchoolRegionHistory(await api<SchoolRegionHistory[]>("list_school_region_history"));
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadRegions = React.useCallback(async () => {
    try {
      setRegions(await api<Region[]>("list_regions"));
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadLectureModels = React.useCallback(async () => {
    try {
      setLectureModels(await api<LectureModel[]>("list_lecture_models"));
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadClassPlans = React.useCallback(async (schoolId?: number) => {
    try {
      setClassPlans(await api<SchoolClassPlan[]>("list_school_class_plans", { schoolId }));
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadProgramDashboard = React.useCallback(async () => {
    try {
      setProgramDashboard(await api<SchoolProgramDashboard>("get_school_program_dashboard"));
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadStudents = React.useCallback(async (schoolId?: number, page = 1, search = studentSearch) => {
    try {
      const result = await api<Paginated<Student> | Student[]>("list_students", {
        schoolId,
        page,
        pageSize: studentPageSize,
        search,
      });
      if (Array.isArray(result)) {
        setStudents(result);
        setStudentTotalCount(result.length);
        setStudentPage(1);
      } else {
        setStudents(result.items);
        setStudentTotalCount(result.total_count);
        setStudentPage(result.page);
      }
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError, studentPageSize, studentSearch]);

  const loadBatches = React.useCallback(async (schoolId?: number) => {
    try {
      setBatches(await api<Batch[]>("list_batches", { schoolId }));
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadSchoolDeleteImpact = React.useCallback(async (id: number) => {
    try {
      const impact = await api<SchoolDeleteImpact>("get_school_delete_impact", { id });
      onError("");
      return impact;
    } catch (caught) {
      onError(String(caught));
      return null;
    }
  }, [onError]);

  // ── Mutations ─────────────────────────────────────────────────────────────

  const createSchool = React.useCallback(async (input: SchoolProfileDraft) => {
    try {
      const school = await api<School>("create_school", { input });
      await loadSchools();
      await loadSchoolRegionHistory();
      await loadProgramDashboard();
      await onLoadAuditLog();
      onNotice(`School profile saved: ${school.name}.`);
      onError("");
    } catch (caught) {
      onError(String(caught));
      throw caught;
    }
  }, [loadSchools, loadSchoolRegionHistory, loadProgramDashboard, onLoadAuditLog, onError, onNotice]);

  const dropSchool = React.useCallback(async (id: number, reason: string) => {
    try {
      const school = await api<School>("drop_school", { id, body: { reason } });
      await loadSchools();
      await loadDroppedSchools();
      setStudents([]);
      setStudentTotalCount(0);
      await loadClassPlans();
      await loadProgramDashboard();
      await onLoadAuditLog();
      onNotice(`School dropped: ${school.name}.`);
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadSchools, loadDroppedSchools, loadStudents, loadClassPlans, loadProgramDashboard, onLoadAuditLog, onError, onNotice]);

  const deleteSchool = React.useCallback(async (id: number) => {
    try {
      await api("delete_school", { id });
      await loadSchools();
      await loadDroppedSchools();
      setStudents([]);
      setStudentTotalCount(0);
      await loadClassPlans();
      await loadSchoolRegionHistory();
      await loadProgramDashboard();
      await onLoadAuditLog();
      onNotice("School master record deleted.");
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadSchools, loadDroppedSchools, loadStudents, loadClassPlans, loadSchoolRegionHistory, loadProgramDashboard, onLoadAuditLog, onError, onNotice]);

  const restoreSchool = React.useCallback(async (id: number) => {
    try {
      const school = await api<School>("restore_school", { id });
      await loadSchools();
      await loadDroppedSchools();
      setStudents([]);
      setStudentTotalCount(0);
      await loadClassPlans();
      await loadProgramDashboard();
      await onLoadAuditLog();
      onNotice(`School restored: ${school.name}.`);
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadSchools, loadDroppedSchools, loadStudents, loadClassPlans, loadProgramDashboard, onLoadAuditLog, onError, onNotice]);

  const saveRegion = React.useCallback(async (input: {
    id?: number;
    name: string;
    regional_academic_head_name: string;
    regional_academic_head_mobile: string;
    regional_academic_head_email: string;
    regional_business_head_name: string;
    regional_business_head_mobile: string;
    regional_business_head_email: string;
  }) => {
    try {
      const region = await api<Region>("upsert_region", { input });
      await loadRegions();
      await onLoadAuditLog();
      onNotice(`Region saved: ${region.name}.`);
      onError("");
    } catch (caught) {
      onError(String(caught));
      throw caught;
    }
  }, [loadRegions, onLoadAuditLog, onError, onNotice]);

  const deleteRegion = React.useCallback(async (id: number) => {
    try {
      await api("delete_region", { id });
      await loadRegions();
      onNotice("Region deleted.");
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadRegions, onError, onNotice]);

  const remapAndDeleteRegion = React.useCallback(async (
    regionId: number,
    mappings: Array<{
      school_id: number;
      target_region_id?: number;
      new_region_name?: string;
    }>,
  ) => {
    try {
      const createdRegions = new Map<string, number>();

      for (const mapping of mappings) {
        const school = schools.find((item) => item.id === mapping.school_id);
        if (!school) continue;

        let targetRegionId = mapping.target_region_id;
        const newRegionName = mapping.new_region_name?.trim() ?? "";

        if (!targetRegionId) {
          if (!newRegionName) {
            throw new Error(`New region name is required for ${school.name}.`);
          }

          const cacheKey = newRegionName.toLocaleLowerCase();
          targetRegionId = createdRegions.get(cacheKey);

          if (!targetRegionId) {
            const region = await api<Region>("upsert_region", {
              input: {
                name: newRegionName,
                regional_academic_head_name: "",
                regional_academic_head_mobile: "",
                regional_academic_head_email: "",
                regional_business_head_name: "",
                regional_business_head_mobile: "",
                regional_business_head_email: "",
              },
            });
            targetRegionId = region.id;
            createdRegions.set(cacheKey, region.id);
          }
        }

        await api<School>("create_school", {
          input: {
            ...schoolToProfileDraft(school),
            region_id: targetRegionId,
          },
        });
      }

      await api("delete_region", { id: regionId });
      await loadRegions();
      await loadSchools();
      await loadSchoolRegionHistory();
      await loadProgramDashboard();
      onNotice("Schools moved and region deleted.");
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [schools, loadRegions, loadSchools, loadSchoolRegionHistory, loadProgramDashboard, onError, onNotice]);

  const createLectureModel = React.useCallback(async (input: {
    name: string;
    days_per_week: number;
    lectures_per_day: number;
  }) => {
    try {
      const model = await api<LectureModel>("create_lecture_model", { input });
      await loadLectureModels();
      onNotice(`Lecture model saved: ${model.name}.`);
      onError("");
    } catch (caught) {
      onError(String(caught));
      throw caught;
    }
  }, [loadLectureModels, onError, onNotice]);

  const saveSchoolClassPlan = React.useCallback(async (input: {
    school_id: number;
    grade_level: string;
    track: string;
    lecture_model_id: number;
    batch_pattern: string;
    aop_admissions: number;
    registrations: number;
    actual_admissions: number;
  }) => {
    try {
      const plan = await api<SchoolClassPlan>("upsert_school_class_plan", { input });
      await loadClassPlans();
      await loadProgramDashboard();
      const trackLabel = plan.track ? ` (${plan.track})` : "";
      onNotice(`Class plan saved: ${plan.school_name} ${plan.grade_level}${trackLabel}.`);
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadClassPlans, loadProgramDashboard, onError, onNotice]);

  const createBatch = React.useCallback(async (input: CreateBatchInput) => {
    try {
      const batch = await api<Batch>("create_batch", { input });
      await loadBatches();
      onNotice(`Batch added: ${batch.batch_id}.`);
      onError("");
    } catch (caught) {
      onError(String(caught));
      throw caught;
    }
  }, [loadBatches, onError, onNotice]);

  const updateBatch = React.useCallback(async (input: UpdateBatchInput) => {
    try {
      const batch = await api<Batch>("update_batch", { input });
      await loadBatches();
      onNotice(`Batch updated: ${batch.batch_id}.`);
      onError("");
    } catch (caught) {
      onError(String(caught));
      throw caught;
    }
  }, [loadBatches, onError, onNotice]);

  const archiveBatch = React.useCallback(async (id: number) => {
    try {
      await api("archive_batch", { id });
      await loadBatches();
      onNotice("Batch archived.");
      onError("");
    } catch (caught) {
      onError(String(caught));
      throw caught;
    }
  }, [loadBatches, onError, onNotice]);

  const createStudent = React.useCallback(async (input: Record<string, unknown>) => {
    try {
      const student = await api<Student>("create_student", { input });
      const schoolId = typeof input.school_id === "number" ? input.school_id : undefined;
      await loadStudents(schoolId, 1);
      onNotice(`Student added: ${student.name}.`);
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadStudents, onError, onNotice]);

  const updateStudent = React.useCallback(async (input: Record<string, unknown>) => {
    try {
      const student = await api<Student>("update_student", { input });
      const schoolId = typeof input.school_id === "number" ? input.school_id : undefined;
      await loadStudents(schoolId, studentPage);
      onNotice(`Student updated: ${student.name}.`);
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadStudents, onError, onNotice]);

  const deleteStudent = React.useCallback(async (id: number) => {
    try {
      await api("delete_student", { id });
      await loadStudents(undefined, studentPage);
      onNotice("Student deleted.");
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadStudents, onError, onNotice]);

  // ── CSV imports ───────────────────────────────────────────────────────────

  const pickFile = React.useCallback((handler: (file: File) => Promise<void>) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".csv,text/csv";
    input.style.display = "none";
    document.body.appendChild(input);
    const cleanup = () => {
      if (input.parentNode) input.parentNode.removeChild(input);
    };
    input.onchange = async () => {
      try {
        const file = input.files?.[0];
        if (file) await handler(file);
      } finally {
        cleanup();
      }
    };
    input.addEventListener("cancel", cleanup);
    input.click();
  }, []);

  const importSchoolsCsv = React.useCallback(() => {
    pickFile(async (file) => {
      try {
        const result = await uploadFile<{
          imported_count: number;
          skipped_count: number;
          errors: string[];
        }>("/imports/schools.csv", file);
        await loadSchools();
        await onLoadAuditLog();
        const errSuffix = result.errors.length > 0 ? ` First error: ${result.errors[0]}` : "";
        onNotice(`Imported ${result.imported_count} schools (${result.skipped_count} skipped).${errSuffix}`);
        onError("");
      } catch (caught) {
        onError(`School import failed: ${caught}`);
      }
    });
  }, [pickFile, loadSchools, onLoadAuditLog, onError, onNotice]);

  const importStudentsCsv = React.useCallback((schoolId: number) => {
    pickFile(async (file) => {
      try {
        const result = await uploadFile<{
          imported_count: number;
          updated_count: number;
          skipped_count: number;
          errors: string[];
        }>("/imports/students.csv", file, { school_id: String(schoolId) });
        await loadStudents(schoolId, 1);
        await loadBatches();
        const errSuffix = result.errors.length > 0 ? ` First error: ${result.errors[0]}` : "";
        onNotice(`Imported ${result.imported_count} students (${result.skipped_count} skipped).${errSuffix}`);
        onError("");
      } catch (caught) {
        onError(`Student import failed: ${caught}`);
      }
    });
  }, [pickFile, loadStudents, loadBatches, onError, onNotice]);

  const importSipMaster = React.useCallback(() => {
    pickFile(async (file) => {
      try {
        const preview = await uploadFile<SipMasterImportPreview>(
          "/imports/sip-master/preview",
          file,
        );
        pendingSipFileRef.current = file;
        setSipImportReview({ sourcePath: file.name, preview });
        onError("");
      } catch (caught) {
        onError(`SIP master preview failed: ${caught}`);
      }
    });
  }, [pickFile, onError]);

  const confirmSipMasterImport = React.useCallback(async (
    conflictAction: "skip_existing" | "update_existing",
  ) => {
    const file = pendingSipFileRef.current;
    if (!file) {
      onError("No SIP master file in memory. Pick the file again.");
      return;
    }
    try {
      const form = new FormData();
      form.append("file", file);
      form.append("conflict_action", conflictAction);
      const token = sessionStorage.getItem("td:token");
      const response = await fetch("/api/imports/sip-master", {
        method: "POST",
        headers: token ? { Authorization: `Bearer ${token}` } : {},
        body: form,
      });
      if (!response.ok) {
        const text = await response.text();
        throw new Error(text);
      }
      const result = (await response.json()) as SipMasterImportResult;
      pendingSipFileRef.current = null;
      setSipImportReview(null);
      await loadSchools();
      await loadRegions();
      await loadClassPlans();
      await onLoadAuditLog();
      onNotice(
        `SIP master imported: ${result.imported_count} new, ${result.updated_count} updated, ${result.skipped_count} skipped, ${result.class_plan_count} class offerings saved.`,
      );
      onError("");
    } catch (caught) {
      onError(`SIP master import failed: ${caught}`);
    }
  }, [loadSchools, loadRegions, loadClassPlans, onLoadAuditLog, onError, onNotice]);

  const cancelSipMasterImport = React.useCallback(() => {
    pendingSipFileRef.current = null;
    setSipImportReview(null);
  }, []);

  // ── Export ────────────────────────────────────────────────────────────────

  const exportSipMasterExcel = React.useCallback(async () => {
    try {
      await api("export_sip_master");
      onNotice("SIP master export started. Check your downloads.");
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError, onNotice]);

  // ── Return ────────────────────────────────────────────────────────────────

  return {
    schools, droppedSchools, regions, lectureModels, classPlans,
    students, studentTotalCount, studentPage, studentPageSize, studentSearch,
    batches, schoolRegionHistory, programDashboard, sipImportReview,
    setSchools, setDroppedSchools, setRegions, setLectureModels, setClassPlans,
    setStudents, setStudentSearch, setBatches, setSchoolRegionHistory, setProgramDashboard, setSipImportReview,
    loadSchools, loadDroppedSchools, loadSchoolRegionHistory, loadRegions,
    loadLectureModels, loadClassPlans, loadProgramDashboard, loadStudents, loadBatches, loadSchoolDeleteImpact,
    createSchool, dropSchool, deleteSchool, restoreSchool,
    saveRegion, deleteRegion, remapAndDeleteRegion,
    createLectureModel, saveSchoolClassPlan,
    createBatch, updateBatch, archiveBatch,
    createStudent, updateStudent, deleteStudent,
    importSchoolsCsv, importStudentsCsv, importSipMaster,
    confirmSipMasterImport, cancelSipMasterImport,
    exportSipMasterExcel,
  };
}
